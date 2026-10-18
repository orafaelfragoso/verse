<!-- spec:begin (copied verbatim into verse-spec.md by build.sh) -->
# The Verse Programming Language

**Language Specification — Version 0.1 (Draft)**

*File extension: `.vs` · Command-line tool: `verse`*

---

## Contents

1. [Philosophy and design principles](01-philosophy.md#1-philosophy-and-design-principles)
2. [Quick tour](02-tour.md#2-quick-tour)
3. [Lexical structure and syntax](03-lexical.md#3-lexical-structure-and-syntax)
4. [Types](04-types.md#4-types)
5. [Functions](05-functions.md#5-functions)
6. [Control flow](06-control-flow.md#6-control-flow)
7. [Pattern matching](07-pattern-matching.md#7-pattern-matching)
8. [Errors and missing values](08-errors.md#8-errors-and-missing-values)
9. [Collections](09-collections.md#9-collections)
10. [Traits and generics](10-traits-generics.md#10-traits-and-generics)
11. [Concurrency](11-concurrency.md#11-concurrency)
12. [Modules](12-modules.md#12-modules)
13. [Resource management](13-resources.md#13-resource-management)
14. [Testing and docs](14-testing-docs.md#14-testing-and-docs)
15. [Standard library overview](15-stdlib.md#15-standard-library-overview)
16. [Tooling](16-tooling.md#16-tooling)
17. [EBNF grammar sketch](grammar.ebnf.md#17-ebnf-grammar-sketch)
18. [Design notes](18-design-notes.md#18-design-notes)
19. [Verse vs other languages](19-comparison.md#19-verse-vs-other-languages)
20. [Complete example programs](20-examples.md#20-complete-example-programs)
21. [Review log and resolved decisions](21-review-log.md#21-review-log-and-resolved-decisions)

---

<!-- spec:end -->

## Section status

| § | File | Status |
|---|------|--------|
| 1 | [01-philosophy.md](01-philosophy.md) | agreed |
| 2 | [02-tour.md](02-tour.md) | agreed |
| 3 | [03-lexical.md](03-lexical.md) | agreed |
| 4 | [04-types.md](04-types.md) | agreed |
| 5 | [05-functions.md](05-functions.md) | agreed |
| 6 | [06-control-flow.md](06-control-flow.md) | agreed |
| 7 | [07-pattern-matching.md](07-pattern-matching.md) | agreed |
| 8 | [08-errors.md](08-errors.md) | agreed |
| 9 | [09-collections.md](09-collections.md) | agreed |
| 10 | [10-traits-generics.md](10-traits-generics.md) | agreed |
| 11 | [11-concurrency.md](11-concurrency.md) | agreed |
| 12 | [12-modules.md](12-modules.md) | agreed |
| 13 | [13-resources.md](13-resources.md) | agreed |
| 14 | [14-testing-docs.md](14-testing-docs.md) | agreed |
| 15 | [15-stdlib.md](15-stdlib.md) | agreed |
| 16 | [16-tooling.md](16-tooling.md) | agreed |
| 17 | [grammar.ebnf.md](grammar.ebnf.md) | agreed |
| 18 | [18-design-notes.md](18-design-notes.md) | agreed |
| 19 | [19-comparison.md](19-comparison.md) | agreed |
| 20 | [20-examples.md](20-examples.md) | agreed |
| 21 | [21-review-log.md](21-review-log.md) | agreed |

Other files: [17-grammar.md](17-grammar.md) (stub pointing to the grammar),
[reviews/](reviews/) (`review-N.md`), [changes/](changes/) (`changes-N.md`),
and [build.sh](build.sh), which concatenates the parts into
`/workspace/verse-spec.md` and renders HTML/PDF.

Implementation architecture (not part of the spec build):
[architecture/README.md](architecture/README.md).
