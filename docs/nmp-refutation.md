# NMP Refutation

Thing I saw in Potential

## Test

https://rektdie.pythonanywhere.com/test/2156/

## Diff

https://github.com/ProgramciDusunur/Potential/compare/266366d4..4ebcdb7f#diff-d5920a54fc6fff9b49052b09ca50c0608cf55cc93b5a46713bf8a4de1132eed0R1182-R1195

## Idea

_Reminder_: If our static eval is good enough (significantly above beta), we try
a reduced search after giving the opponent a free move. If, after the reduced
search, we _still_ end up with a beta cutoff, we prune this branch and return
the score from the reduced search.

The refutation idea, then, is that if the reduced search comes back with a score
_less_ than beta, then the opponent managed to find a pretty decent refutation
move. We probably want to give that move special consideration.

In Potential's case, he gives the move a history bonus. Could also make it some
kind of refutation move that we play ahead of everything else?
