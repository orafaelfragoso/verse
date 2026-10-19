#!/usr/bin/env bash
# Applies the repository's GitHub configuration: merge settings, labels,
# milestones (from docs/roadmap.md) and the ruleset on the default branch.
# Rerunning it converges to the same state. CONTRIBUTING.md explains each part.
# Usage: tools/github/setup.sh <owner/repo>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: ${0##*/} <owner/repo>" >&2
  exit 2
fi
repo=$1
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

echo "merge settings"
gh api -X PATCH "repos/$repo" \
  -F allow_squash_merge=true -F allow_merge_commit=false -F allow_rebase_merge=false \
  -f squash_merge_commit_title=PR_TITLE -f squash_merge_commit_message=PR_BODY \
  -F delete_branch_on_merge=true -F allow_auto_merge=true -F allow_update_branch=true \
  >/dev/null

# name|color|description. Any label not listed here is deleted.
labels=(
  "bug|d73a4a|Behaves differently from the spec or the docs"
  "feature|a2eeef|Implements something the docs already specify"
  "design|5319e7|Changes the language, the standard library or the architecture"
  "docs|0075ca|Documentation only"
  "perf|fbca04|Speed, memory or binary size"
  "chore|cfd3d7|Build, CI, dependencies, repository upkeep"
  "area:spec|c5def5|docs/language: the language specification"
  "area:compiler|c5def5|Parser, type checker, ownership, MIR, backends"
  "area:vm|c5def5|Bytecode and the tier-0 VM"
  "area:runtime|c5def5|Runtime slices, tasks, scheduler"
  "area:std|c5def5|Standard library modules"
  "area:cli|c5def5|The verse command and verse.toml"
  "area:tooling|c5def5|Formatter, language server, debugger, docs generator"
  "area:packages|c5def5|Registry, lock file, verse.api"
  "area:plugins|c5def5|extern blocks and native libraries"
  "area:targets|c5def5|Targets, ABI, platform layers, embedded"
  "status:ready|0e8a16|Fully specified: anyone, human or agent, can start"
  "status:needs-design|d4c5f9|Needs a decision before work starts"
  "status:blocked|b60205|Waiting on another issue or an outside change"
  "good first issue|7057ff|Small and self-contained"
)

echo "labels"
wanted=()
for entry in "${labels[@]}"; do
  IFS='|' read -r name color description <<<"$entry"
  wanted+=("$name")
  gh label create "$name" -R "$repo" --color "$color" --description "$description" --force >/dev/null
done
while IFS= read -r name; do
  [[ -z $name ]] && continue
  keep=false
  for w in "${wanted[@]}"; do [[ $w == "$name" ]] && keep=true; done
  if ! $keep; then
    echo "  delete $name"
    gh label delete "$name" -R "$repo" --yes >/dev/null
  fi
done < <(gh label list -R "$repo" --limit 500 --json name --jq '.[].name')

# One milestone per row of the roadmap table: | M0 | Name | Deliverables | Exit criteria |
echo "milestones"
trim() { sed 's/^ *//; s/ *$//' <<<"$1"; }
existing=$(gh api "repos/$repo/milestones?state=all&per_page=100" --jq '.[] | "\(.number)\t\(.title)"')
while IFS= read -r row; do
  fields=$(awk -F'|' '{print NF}' <<<"$row")
  if [[ $fields -ne 6 ]]; then
    echo "roadmap row does not have 4 columns: $row" >&2
    exit 1
  fi
  IFS='|' read -r _ id name deliverables exit_criteria _ <<<"$row"
  title="$(trim "$id"): $(trim "$name")"
  description="Deliverables: $(trim "$deliverables"). Done when: $(trim "$exit_criteria")."
  number=$(awk -F'\t' -v t="$title" '$2 == t {print $1}' <<<"$existing")
  if [[ -n $number ]]; then
    gh api -X PATCH "repos/$repo/milestones/$number" -f description="$description" >/dev/null
  else
    gh api "repos/$repo/milestones" -f title="$title" -f description="$description" >/dev/null
  fi
  echo "  $title"
done < <(grep -E '^\| M[0-9]+ \|' "$root/docs/roadmap.md")

# Changes reach the default branch only through a squash-merged pull request
# whose title passed the commit-format check. Admins may override a pull
# request's checks, never push directly.
echo "ruleset"
ruleset=$(jq -n '{
  name: "default-branch",
  target: "branch",
  enforcement: "active",
  conditions: {ref_name: {include: ["~DEFAULT_BRANCH"], exclude: []}},
  bypass_actors: [{actor_id: 5, actor_type: "RepositoryRole", bypass_mode: "pull_request"}],
  rules: [
    {type: "deletion"},
    {type: "non_fast_forward"},
    {type: "required_linear_history"},
    {type: "pull_request", parameters: {
      required_approving_review_count: 0,
      dismiss_stale_reviews_on_push: false,
      require_code_owner_review: false,
      require_last_push_approval: false,
      required_review_thread_resolution: true,
      allowed_merge_methods: ["squash"]}},
    {type: "required_status_checks", parameters: {
      strict_required_status_checks_policy: false,
      required_status_checks: [{context: "pr-title", integration_id: 15368}]}}
  ]}')
id=$(gh api "repos/$repo/rulesets" --jq '.[] | select(.name == "default-branch") | .id')
if [[ -n $id ]]; then
  gh api -X PUT "repos/$repo/rulesets/$id" --input - <<<"$ruleset" >/dev/null
else
  gh api "repos/$repo/rulesets" --input - <<<"$ruleset" >/dev/null
fi
echo "done"
