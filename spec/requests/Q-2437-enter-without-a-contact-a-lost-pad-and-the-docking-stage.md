---
id: Q-2437
title: Enter without a contact - a lost pad, the move on to the docking stage and its fall-back
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r103 (Q-2366) claim 24 makes the queued behaviour the only condition of the wait in a
processing of *enter* without a contact: "With a behaviour queued, it keeps the destination and
*enter*, asks no pad and waits ... The wait is the same when the order that queued the behaviour
also gave the aircraft a destination other than the pad, such as a cell." It adds that "a
processing without a contact that finds the aircraft landing also moves it on to the docking
stage", and that in the docking stage without a contact "the first processing that finds it no
longer landing makes it fall back to rest (claim 18), whatever is queued". Our engine has to
decide four points the text leaves open (`air::without_contact`, `air::docking_without_contact`):

- r99 ended the wait with a fall-back when the destination was no longer a pad (gone, being
  sold, or not a building of the aircraft's dock types), citing claims 32 and 46. Claim 32 says
  "a surviving approaching aircraft whose contact or destination is lost falls back to rest";
  claim 24's sale sentence speaks of an aircraft that "approaches". It is not clear whether an
  aircraft in *enter* without a contact counts as approaching there, so whether a lost pad still
  ends the wait when a behaviour is queued.
- "Finds the aircraft landing": anywhere, or only over a cell of its destination pad's
  foundation, as the processing in contact advances only there?
- The docking stage's fall-back "whatever is queued": does it forget the destination first, as
  the processing without a contact with nothing queued does, or fall back with the pad (or, after
  an accepted touchdown, no destination) as it stands?
- A pad being sold while the aircraft is in the docking stage, in contact: is that the sale of
  claim 24 ("if the pad stops being its destination while it approaches, a sale ... it falls back
  to rest"), or a refusal that leaves it to the docking stage's rules without a contact?

## Questions

1. An aircraft in *enter* without a contact, a behaviour queued, whose destination pad is
   destroyed, removed, or being sold (or is a building that is not one of its dock types): at its
   next processing of *enter*, does it wait (the queued behaviour current at the end of that tick)
   or fall back to rest?
2. An aircraft in *enter* without a contact that is landing over a cell that is not in its
   destination pad's foundation (or whose destination is not a building) at a processing of
   *enter*: is it moved on to the docking stage?
3. In the docking stage without a contact, at the first processing that finds the aircraft no
   longer landing: is its destination forgotten before the fall-back to rest, or does the
   fall-back see the destination it has then?
4. An aircraft in the docking stage, in contact, whose pad is being sold at its next processing
   of *enter*: does it fall back to rest at once, or, while landing, go on with the docking stage's
   step without a contact until it is no longer landing?

## Possible observation

S-0501 AC31's harness and part f's set-up. For 1: queue a behaviour without an immediate start
before the second processing of *enter* and sell the pad in the same tick. For 2 to 4: the same
harness with a test copy of the pad whose sale or refusal can be started at a chosen tick while the
aircraft lands over it. Record per tick the aircraft's behaviour (current and queued),
destination, contact, height and position.

## Current workaround

Question 1: a lost pad (gone, being sold, or not of its dock types) ends the wait with a fall-back
that forgets the destination, as in r99. Question 2: the move on to the docking stage needs a cell
of the destination pad's foundation, as the processing in contact does. Question 3: the fall-back
sees the destination as it stands (claim 24 says only "fall back to rest"). Question 4: a sale
falls back at once, before the docking stage's rules, as in r99.
