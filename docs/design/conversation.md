# Conversations and interiority: proposal

2026-10-24. The first spike is built (see "Status" at the end). Direction
from Ian over several messages (see the decision log); background in
`docs/research/conversation-frames.md` and
`docs/research/interiority-psychology.md`.

## Principles

- **Interiority is a program per character.** Each villager has a
  self-concept and a concept of each person they know, and their own code
  decides how events change those and how they show in behaviour. Shared
  defaults cover ordinary human responses; a character overrides only what
  is theirs, as with needs today.
- **Warmth belongs to conversations, not to pairs.** Nothing stores "warmth
  between Tom and Hal". A conversation has observable parts, each
  participant reads it their own way, and what they keep is a memory of the
  episode. How someone feels about another person is read from memory by
  their program when needed.
- **A conversation runs in a negotiated frame.** The frame (buying bread,
  small talk, confiding) says which moves make sense and what they mean.
  Opening or shifting a frame is a bid the other person answers.
- **Intent, performance and uptake can each come apart.** A flirt can come
  out as an insult, or be heard as one. Both are inside the frame; they are
  landing wrong, not breaking convention.
- **Breaking frame is a distinct event with its own handler.** Declaring
  love to the cashier steps outside the frame. Villagers rarely do it; a
  player will do it constantly. Every character implements a response to
  it, with a shared default.
- **Characters know each other only through perception.** No character code
  reads another character's private state. Mara thinks Tom seems low because
  she saw him drinking alone, and she can be wrong. (Today Mara reads
  `tom.mood` directly; that goes.)

## Frames

A first set covering phatic contact, scripted exchange and Fiske's four
relational models. Frames are shared: they are the town's conventions.

| Frame | Relational model | Usual setting | In-frame moves | Usual bids out |
|---|---|---|---|---|
| greeting | phatic | passing outdoors | greet, acknowledge | small talk |
| transaction | market pricing | counter, bar | ask, offer, pay, thank, haggle | small talk |
| small talk | phatic, equality | anywhere | tell, opine, joke, agree, ask | catching up, courting, confiding, favour |
| catching up | equality matching | tavern, bench, doorstep | tell, ask, opine, gossip, agree, disagree | confiding, quarrel |
| favour | equality matching | anywhere | request, offer, promise, thank, decline | transaction, quarrel |
| confiding | communal sharing | somewhere private | disclose, sympathise, advise, comfort | small talk (to close) |
| courting | communal, play | private or half-private | compliment, tease, invite, disclose | confiding |
| quarrel | any, broken down | anywhere | complain, accuse, justify, apologise, retort | confiding (making up), ending |

Authority ranking (instruction, deference) has no frame yet because the town
has no hierarchy; it comes with employers, family elders or officials.

## Moves

A small general vocabulary (Searle's speech acts plus Bales's emotional
reactions): greet, acknowledge, tell, ask, opine, agree, disagree, joke,
tease, compliment, complain, request, offer, promise, thank, apologise,
sympathise, disclose, gossip, accuse, decline, pay. A move can carry a
subject (a person, a fact, an object, a topic).

The frame gives each move its conventional meaning. A compliment at the
counter is politeness; in courting it is interest. A tease costs face, and
what it costs depends on how well the two know each other (Brown and
Levinson), so the same tease can warm a friend and offend a stranger.

## Negotiation

A conversation opens with a frame one person proposes, usually the setting's
default. Either participant can bid to shift it. The other answers (Gottman's
terms):

- **toward:** accept; the frame shifts.
- **away:** ignore or deflect; the frame stays. With an account ("the oven's
  on") it reads differently from without one.
- **against:** rebuff; usually the conversation sours or ends.

Bids and answers are where stories start: she tried to make it personal and
he kept it about bread.

## Intent, performance, uptake

For every move the record keeps three things:

1. **Intent:** what the speaker's program chose (tease).
2. **Performance:** what came out, from their skill plus a roll from their
   mind stream (a jab).
3. **Uptake:** what each hearer took it as, from the hearer's program and
   their view of the speaker (an insult). Bystanders in earshot see only the
   performance and form their own uptake.

Misfires in performance and misreadings in uptake are different failures and
both get recorded, because neither can be recovered later.

## Breaking frame

A breach is a move the current frame does not contain, made knowingly or
obliviously: the wrong relational model (offering money for a favour), the
wrong intimacy (confiding at the counter), or a convention ignored outright.
The default response follows Garfinkel's breaching experiments: try to make
sense of it ("are you joking?"), then repair or re-key it (treat it as a
joke, change the subject), then sanction it (awkwardness, offence, telling
others). Witnesses remember breaches strongly.

Each character implements `out_of_frame` as a standard part of their
program, even if it only adjusts the default: who laughs it off, who is
scandalised, who plays along.

## The inner model, grounded in psychology

What each villager carries, and the research behind the defaults:

- **Personality is the program.** Per-character code is a set of if-then
  signatures (Mischel and Shoda): "if someone I'm not easy with starts small
  talk, I get anxious". Traits like introversion are parameters to the
  shared defaults, not the personality itself. Moment-to-moment stance
  varies around the person's centre, drawn from their own stream (Fleeson).
- **Self-concept:** a few central beliefs ("I'm reliable"), hoped-for and
  feared selves, and the gaps between them. A failed hope saddens, a failed
  duty agitates (Higgins, as a default). Self-views can be wrong and resist
  correction (Swann). Being turned away lowers felt standing (Leary).
- **Views of others:** impressions built by attribution. The default
  explains others' behaviour by character and one's own by circumstance
  (fundamental attribution error, actor-observer asymmetry). "What do they
  think of me?" defaults to one's own self-view, nudged by what the other
  actually did (Kenny and DePaulo).
- **Sensitivities:** rejection sensitivity and attachment style as
  parameters that change how bids are made and answers are read (Downey and
  Feldman; Hazan and Shaver).
- **Feelings:** appraisal of what happened against the person's goals,
  standards and attitudes (OCC, Lazarus), coloured by mood (Forgas), and
  partly caught from the room (contagion).
- **Warmth:** made by disclosure answered with responsiveness, as the
  discloser perceives it (Reis and Shaver), deepening gradually and needing
  to be matched (Altman and Taylor; Aron). Familiarity also grows just from
  crossing paths (Festinger et al.; Zajonc).
- **Memory:** an episode is kept by its peak and its end, and bad weighs
  more than good (Kahneman; Baumeister), unless the person's program
  forgives.

## What a character's program provides

Each has a shared default in the toolbox, the way `toolbox::human` answers
needs today:

| Hook | Question it answers |
|---|---|
| `stance` | How do I come into this, given who it is, where, who's watching, my mood? Default: warmth and dominance complement the other's (interpersonal circumplex), varied around my centre. |
| `propose` | What frame do I open with, if any? Default: the setting's frame, deeper only with history (social penetration). |
| `answer_bid` | Toward, away (with or without an account), or against? Default: toward when it matches how close we are and my mood allows. |
| `next_move` | What do I mean to do next? Default: in-frame moves serving my current goal; match a disclosure. |
| `perform` | How well does it come out? Default: skill plus a roll; teasing and flirting are the hardest to land. |
| `take_up` | What do I take their move as, and how does it make me feel? Default: attribution to character, appraisal against my goals and standards, coloured by mood and rejection sensitivity. |
| `out_of_frame` | What do I do when they step outside? Default: make sense, repair, sanction (Garfinkel). |
| `remember`, `recall` | What do I keep from this, and what do I make of them now? Default: peak and end, bad weighted more. |

A typical character writes three or four of these: the shy one's `stance`
and `take_up`, the gossip's `next_move`, everyone's `out_of_frame`.

## In the sim

- A conversation is a short sequence of beats run by the sim: open frame,
  then a few moves and bids, then close. Each beat is an event, and each
  roll comes from the speaker's own stream, so it replays exactly.
- New events, each with a cause: conversation started (participants, frame,
  setting, private or overheard), bid and answer, move (intent and
  performance), uptake (per hearer, including bystanders), breach,
  conversation ended. The current `Talk` command becomes "start a
  conversation".
- Memories are episodes kept in each character's own state, pruned by their
  own program (some people forget slights, some don't).
- Summaries report what deviates: a bid turned away by someone who usually
  turns toward, a misreading, a breach, a cold conversation between two who
  are usually warm. Each line replays to the moment.
- The inspector shows a person's current stance, recent uptakes and what they
  make of whoever they are talking to.

## First spike

- Frames: greeting, transaction, small talk, confiding. Moves: about ten.
- Shared defaults for every hook. Personal programs for two or three
  placeholder people, for Ian to rewrite.
- Remove direct reads of other characters' private state.
- Acceptance: on some seed, the summaries can tell a scene like this one,
  and clicking it replays it. Hal buys bread at Mara's counter. Mara bids
  for small talk about the road and Hal turns toward it. Hal bids to confide
  his money trouble; Mara, with Nell waiting behind him, turns away with an
  account, meaning discretion. Hal takes it as coldness. Nell overheard the
  word "money".

## Open questions

- Should characters be able to be wrong about themselves? The research says
  people routinely are, and that their beliefs about how others see them
  mostly mirror their self-view. Leaning: yes, self-concept is a belief.
- Norms: shared only, or can each character judge breaches by their own
  standards? Leaning: frames shared, judgement personal.
- How many beats in a conversation, and does length matter to uptake?
- Frame and move names are engineering vocabulary. Anything a player reads
  from them later needs Ian's voice.

## Status

The first spike is in. Where things are:

- `crates/sim/src/social.rs`: persona, memory, regard, the `Interlocutor`
  trait and its research defaults.
- `crates/sim/src/sim/conversation.rs`: conversations as beats, bystanders,
  frame changes, episodes.
- `crates/characters/src/*.rs`: one placeholder program per villager.
- `crates/sim/src/summary/talk.rs`: the summary line for a conversation.
- `cargo run -p tools -- talks --seed 2 --day 36 --who mara` prints
  transcripts; the viewer's inspector shows a villager's mind.

Found while tuning:

- Doubling bad episodes by value made one mildly bad chat read as cold.
  Bad episodes now carry double weight in the mean instead, and the mean is
  rounded.
- Counting a counter's scripted thanks toward how someone seemed hid every
  low mood. Only unscripted moves count now.
- Without a gap before retelling, a worried villager confided in everyone
  every day. Now it is a week per person, and only with people they like.
- Transactions misread as prying were noise; scripted moves are now read
  worse a quarter as often.

Not done yet: catching up, favour, courting and quarrel frames; the
self-concept as a belief; anyone but Mara having a worry often enough to
confide it; summaries that say why a misreading happened.

Since then (2026-10-25): moves are called gestures in the code
(`Gesture`), each character has an exhaustive table of what each gesture
means to them, and topics are declared in `content/topics/`. See the
decision log.
