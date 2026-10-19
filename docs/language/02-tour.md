## 2. Quick tour

The following program shows most of Verse on one page. It reads a list of
people, greets them, and summarizes some facts.

```verse
## A small tour of Verse.

import read_text from fs

type Person
    name: Text
    age: Int
    email: Text? = none

type Shape = Circle(radius: Float) | Square(side: Float)

# Types are optional. This function is fully inferred…
fn greet(person, greeting = "Hello")
    "{greeting}, {person.name}!"

# …and this one spells its types out. Both are statically checked.
fn area(shape: Shape): Float
    match shape
        Circle(r) -> 3.14 * r * r
        Square(s) -> s * s

fn parse_age(text: Text): Int raises Error
    age = try text.trim().to_int()
    if age < 0
        return error("age cannot be negative: {age}")
    age

# `mut` borrows exclusively; `take` consumes the argument.
fn add_guest(mut list, take guest)
    list.push(guest)

mut people = [
    Person(name: "Ana", age: 30, email: "ana@example.com"),
    Person(name: "Bruno", age: 17),
]
add_guest(mut people, Person(name: "Carla", age: 41))

for person in people                      # each person is a value of its own
    print(greet(person, greeting: "Welcome"))
    print("  contact: {person.email otherwise "no email"}")

adults = people.filter(p -> p.age >= 18).map(.name)
label = if adults.len() == 1 then "adult" else "adults"
print("{adults.len()} {label}: {adults.join(", ")}")

bruno_email = people.find(p -> p.name == "Bruno").email otherwise "none"
print("Bruno's email: {bruno_email}")

total = [Circle(radius: 1.0), Square(side: 2.0)].map(area).sum()
print("Total area: {total:.2}")

mut count = 0                             # `x = …` binds once; `mut` can change
for i in 1..=10
    if i % 2 == 0
        count += 1
print("{count} even numbers between 1 and 10")

mut first = "Ana"
mut second = "Bruno"
(first, second) = (second, first)         # both are `mut`: a swap
print("{first}, then {second}")

config = spawn read_text("config.txt")
print(config.wait() otherwise "no config found")

test "greeting uses the given name"
    ana = Person(name: "Ana", age: 30)
    assert greet(ana) == "Hello, Ana!"

test "negative ages are rejected"
    assert parse_age("-3") is _: Error
    assert try parse_age(" 42 ") == 42
```

Run it with `verse tour.vs`; run its tests with `verse test tour.vs`.
Two things are worth noticing. `greet` and `add_guest` have no type
annotations at all, yet everything is checked at compile time (§4.2). And no
line shows ownership: `adults` is a list of its own, and the names in it share
their storage with the people they came from until one of them changes, so
nothing is duplicated (§4.9, §9.5).

---

