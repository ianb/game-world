# A triangle among the cast

Status: built 2026-10-25. Ian asked for social tension ("love triangle or
something") and chose to use the existing villagers. The casting is mine
and stays open to change: Hal is sweet on Mara, Mara is fond of Tom, and
Tom does not notice.

## Pieces

**Topics between two people.** `Topic` has a second person, `with`, so
"sweet on (Hal, Mara)" is one topic: about Hal, with Mara. Labels show
both names. Hearing either name counts as hearing yourself mentioned.
Gossip never tells the person a topic is about. It tells the `with`
person only at discretion 2 or less, which in this cast means Pip.

**Mara.** Evenings at the tavern, she goes back to the bar for another
word with Tom whenever they have not talked for an hour, and in small
talk with him she compliments him. Once she has heard in words that Hal
is sweet on her, either told so or catching him muttering about a rival,
she keeps him at a distance. She serves him and thanks him at the
counter but turns away his bids and leaves small talk. This is a
`knows_hal_is_sweet` flag, set once.

**Hal.** On days off the road he sometimes calls at the bakery for a
word. He watches whom Mara seeks out: a conversation she starts that is
not business at a counter, and that he saw (`toolbox::seen_seeking_out`
reads the town's record). Each one raises his jealousy and names that
person his rival. Jealousy falls a point a day and two after a warm talk
with her. Once it is high enough:

- after a drink, the thought comes out aloud, as a complaint about the
  rival, and anyone near enough catches it;
- with the rival he takes digs;
- with someone who is neither of them, he bids to confide that he is
  sweet on Mara.

**Tom.** Overhearing a complaint about himself adds to his grievance and
a thought against the speaker. He does not wonder why.

## What it did

Six seeds, six weeks:

| seed | Mara finds out | Hal aloud | digs at Tom | Tom stung | retold |
| ---- | -------------- | --------- | ----------- | --------- | ------ |
| 1    | day 12         | 6         | 7           | 4         | 6      |
| 2    | day 19         | 4         | 3           | 3         | 0      |
| 3    | day 12         | 5         | 2           | 4         | 3      |
| 4    | day 5          | 4         | 1           | 3         | 0      |
| 5    | day 12         | 6         | 3           | 6         | 0      |
| 6    | day 5          | 10        | 3           | 5         | 6      |

Mostly Mara finds out by overhearing Hal mutter at the next table. Seed
6 shows the gossip route as well. Mara has overheard Hal on Saturday
night. On Sunday at 11:11 Hal, meeting Pip in the square, bids to confide
that he is sweet on Mara. Pip ends the talk without answering, but he has
the words: he tells Nell at 11:17, Mara at 12:14 and Tom at 6:05pm.
Mara, a few tiles off, caught the tone of Hal's bid. The next morning he
tries again, and this time she catches the words.

Once Mara keeps her distance, Hal's talks with her go badly, so his
touchiness stays near its maximum. Nobody wrote that link; it comes from
the rule that bad talks make him touchier.

## Things learned on the way

- At first Hal bought a loaf on each call. That kept Mara solvent, so
  her money worry never arose, and with it went every confiding bid in
  the town and all of Nell's gifts. He now calls without buying. The
  economy is small enough that one habit moves everything.
- Tom's gesture table rates a complaint at zero, because he grumbles
  himself. That table is for what is said to his face, so for a remark
  about him the shared default applies.
- The summary folds repeated thoughts into habits, which hid the first
  time a thought was said aloud. Aloud and silent are now separate
  habits.

## Open

- The arrangement is code in each character's file, not content. If
  more of these appear, who is fond of whom may want to be data.
- Mara's knowing is permanent, and nothing ever resolves it: no
  confession, no falling out, no Hal giving up.
- Tom is oblivious by omission, not by any rule. He could notice Mara's
  compliments, or hear the gossip and do something with it.
- Hal's bids to confide are mostly turned away. The news travels
  anyway, because a bid carries its topic in words.
- The glosses "sweet on" and "a rival" are labels for the summaries and
  viewer. Ian may want different words.
