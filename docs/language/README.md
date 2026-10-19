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
17. [EBNF grammar sketch](17-grammar.md#17-ebnf-grammar-sketch)
18. [Design notes](18-design-notes.md#18-design-notes)
19. [Verse vs other languages](19-comparison.md#19-verse-vs-other-languages)
20. [Complete example programs](20-examples.md#20-complete-example-programs)

---

<!-- spec:end -->

`tools/docs/build.sh` concatenates these chapters into `dist/verse-spec.md` and
renders HTML and PDF. How Verse is built: [architecture](../architecture/README.md).
