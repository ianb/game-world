# Knowledge into action

Status: built 2026-10-25, while Ian was away ("experiment and make
conclusions and act on them"). This note says what was built, what it
showed, and what it leaves open.

## The question

Ian asked whether Nell, or anyone, can act on what they know, and pointed
out that every behavior is programmed specifically. His worry was the
usual design for this: memories get folded into numbers, the numbers feed
logic, and the logic produces action, so by the time anyone acts the
reason is gone.

## What was built

Three pieces, each small.

**Shared rules over memories.** A rule is a plain function in
`worlds/valley/characters/src/toolbox.rs` that reads a character's heard
memories and returns commands. Any character can run it; whether they do,
and how readily, comes from their persona. The first rule is `kindness`:
if you heard, in words and in the last week, a topic that content marks
as trouble (`money`, `going-without`) about someone you like, you are warm
by nature, and you have coins to spare, you give them a few coins. Once
per piece of news. The rule reads the `Heard` entry itself, not a number
derived from it.

**Actions cite their cause.** `Command::Because { from, topic, heard_at }`
sets the cause of the next event to `Cause::Knew`, so the `Paid` event
records who said what, and when. The summary line spells out the chain:
"Nell gave Mara 3 coins: Pip had told Nell about money (Mara) (11:22am)".
The viewer shows the cause in the inspector and on summary items. A test
(`worlds/valley/characters/tests/knowledge.rs`) checks that every cited turn
happened and that the giver caught the words.

**News moves.** `default_gossip` (in `crates/sim/src/social.rs`) is the
default for what someone says when a conversation opens or small talk
needs a topic: something they heard about a third person in the last
week, not a confidence (unless they are indiscreet), not already told to
this listener, and not something the listener has been heard talking
about. How often they pass it on comes from `Persona.discretion`: Pip (1)
tells nearly everything, Mara (9) almost nothing. Pip also stops for a
short chat when he meets someone, which is how most news leaves the
bakery. A `told {topic}` note on the beat lets the summary say "Pip told
Hal about money (Mara)".

## What it showed

Six seeds, six weeks each, after the already-knows filter:

| seed | news passed on | gifts citing news |
| ---- | -------------- | ----------------- |
| 1    | 26             | 3                 |
| 2    | 27             | 2                 |
| 3    | 29             | 1                 |
| 4    | 31             | 4                 |
| 5    | 34             | 0                 |
| 6    | 33             | 1                 |

Before the filter, news was repeated to people who plainly knew it (36 to
55 per seed). One chain on seed 1, day 15, came out of pieces that were never written
to fit together: at the tavern at 7:23pm Mara bids to confide in Tom
about money, Tom turns away, Nell is sitting close enough to catch the
words, and at 8:17pm Nell gives Mara three coins.

Every gift so far is Nell to Mara: Nell is the only warm
character with spare coins who likes Mara, and Mara is the only one whose
trouble gets talked about. The sources vary: Mara herself in the shop, or
Pip passing it on.

## Conclusions

- A rule that reads specific memories and returns commands does the job
  Ian was worried numbers would do, and keeps the reason. Numbers are
  still useful for aggregates that really are aggregates (feeling,
  familiarity), and as thresholds on a rule, not as the carrier of a
  fact.
- Rules shared across characters, gated by persona, give variety without
  writing each behavior per character. The per-character code stays for
  what is specific (Mara's pastry for Tom, Hal's touchiness).
- Topics marked as trouble in content are enough for one rule. A second
  rule will tell whether the marking should stay a closed enum
  (`Trouble`) or grow.
- With five villagers the chains are short and the same few repeat. More
  people, or more kinds of trouble, are what would make them vary.

## Open

- Only one action rule. Candidates: avoiding someone you heard was cross
  with you, checking on someone who tripped, asking the subject directly.
- News is passed on verbatim. There is no distortion yet, which is where
  Ian's "opportunity for misinterpretation" would come in: a gist caught
  from afar could be retold as words.
- Gossip has no cost. Mara never learns that Pip spread her trouble,
  though the record could tell her.
