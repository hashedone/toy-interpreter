I did couple changes (clarifications?) in this language comparing to kata, just to make the language more consistent.

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
