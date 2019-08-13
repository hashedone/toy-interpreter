I did couple changes (clarifications?) in this language comparing to kata, just to make the language more consistent.

Usage: just `cargo run` and put expressions. No scripts implemented, no session saving. It is just a toy excercise, because why not.

## Assignment
It is clear, that assignment itself is an expression and can be chained like:
```
a = b = 3
```

Also if `b` wasnt assigned before, this is pretty obvious:
```
a = 3 + b = 4
```

because `3 + b` is not valid expression. However, if `b` was assigned before, `3 + b` becomes valid on itself, and due to operator precedense it should be calculated before considering before assignment, but it creates semanticaly invalid code, because result of `3 + b` is not valid left side of assignment. According to comments on Kata, this should throw an error, but I don't like this, so I made not-symetrical precedense of `=` - it has the lower precedense that everything on right side, but higher than everything on left side.

## Function call from another function

There is not clear if function may be called from another function. Logically it should be, but there is a problem: functions may be overwritten, with change of their signature (or arity), which may cause functions which were calling it before possibly invalid.

On the other hand if functions would be treaten in same way as variables, its clear, that functions may not call other function, because they cannot be passed in arguments (functions aren't values in this language).

From comments on Kata its clear, that there are no tests calling function from another functions, so secons approach would make things easy, but I don't like this approach - it is inconvinient. I decided to just inline all functions called from other functions.

# Syntax

## Expressions:
```
2 + 3
= 5
4 - 3
= 1
2 * 0.75
= 1
3 / 2
= 1
3 % 2
= 1
```

## Variables
```
a = 4
= 4
a
= 4
a + 1
= 5
a
= 4
a = a + 1
= 5
a
= 5
```

## Functions
```
add a b => a + b
()
add 1 2
= 3
add add 1 2 3
= 6
add3 a b c => add add a b c
= ()
add3 1 2 3
= 6
```
