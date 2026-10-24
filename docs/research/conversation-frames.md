# Conversation frames: what the research offers

2026-10-24. Background for `docs/design/conversation.md`. The question: what
kinds of interaction exist, how do two people settle which one they are in,
how does a move come to mean something other than what was intended, and what
happens when someone steps outside the agreed kind. Sources are classic and
well known; the two game systems were checked against their papers and the
developers' own writing.

## Frames and scripts

- **Goffman, *Frame Analysis* (1974).** A frame is the shared answer to "what
  is going on here?". The same act means different things under different
  frames, and a frame can be re-keyed (a fight becomes play-fighting).
  "Breaking frame" is acting in a way the frame cannot absorb. In *Forms of
  Talk* (1981) "footing" is the stance a participant takes within a frame,
  which shifts more often than the frame does.
  *Gives us:* the frame as the unit both people negotiate; footing as the
  per-person stance inside it; breaking frame as its own event.
- **Schank and Abelson, scripts (1977).** Stereotyped sequences for recurring
  situations, the restaurant script being the famous one: roles, props,
  entry conditions, expected scenes.
  *Gives us:* setting-bound frames such as buying bread at a counter, where
  the steps are known and deviation is noticeable.
- **Versu (Evans and Short, 2014).** Social practices model recurring social
  situations and are presented as successors to the Schankian script. They
  are reactive joint plans that offer affordances to the participants; they
  never control agents, and each agent picks its own action by utility.
  ([paper](https://www.cs.uky.edu/~sgware/reading/papers/evans2014versu.pdf))
  *Gives us:* the split we want. The practice (frame) says what can be done;
  the character decides what to do.

## Kinds of relation

- **Fiske, relational models theory (1992).** Four elementary ways people
  relate: communal sharing (what's mine is ours), authority ranking
  (deference and command), equality matching (turn-taking, favour for
  favour) and market pricing (exchange at a rate). Most interactions run on
  one of them, and using the wrong one is a recognisable offence, such as
  offering to pay for a family dinner.
  *Gives us:* a check that the frame set covers the four, and a ready
  definition of one kind of breach: the wrong model for the moment.
- **Malinowski, "phatic communion" (1923).** Talk whose purpose is the
  contact itself: greetings, weather, how-are-you.
  *Gives us:* greeting and small talk as frames in their own right, with
  little content and real social weight.

## What a move is

- **Searle's classification of illocutionary acts (1976).** Assertives
  (telling), directives (asking, requesting), commissives (offering,
  promising), expressives (thanking, apologising, complaining) and
  declarations.
  *Gives us:* a small, general vocabulary of moves that most frames reuse.
- **Bales, Interaction Process Analysis (1950).** Twelve observation
  categories split between task (giving and asking for information,
  opinions, suggestions) and social-emotional reactions (solidarity,
  tension release, agreement, and their negative counterparts).
  *Gives us:* the emotional moves Searle lacks, such as joking to release
  tension and showing antagonism.
- **Brown and Levinson, politeness (1987).** Face: the wish to be approved
  of (positive face) and not imposed on (negative face). Many moves threaten
  face (requests, criticism, teasing), and speakers soften them with
  politeness strategies whose cost depends on distance, power and the size
  of the imposition.
  *Gives us:* why the same tease lands with a friend and offends a stranger,
  and why a request needs more softening the less well people know each
  other.

## Bids and their answers

- **Gottman, bids for connection.** A bid is any attempt to get attention,
  affirmation or affection. The other person turns toward it (engages),
  turns away (ignores or brushes it off) or turns against it (hostile).
  Repeated turning away makes people stop bidding.
  ([Gottman Institute](https://www.gottman.com/blog/turn-toward-instead-of-away/))
  *Gives us:* the negotiation step. Opening a frame or shifting to another
  is a bid, and these three answers are the responses.
- **Conversation analysis: adjacency pairs and preference (Schegloff and
  Sacks 1973; Sacks, Schegloff and Jefferson 1974).** Talk comes in pairs
  (offer and acceptance, question and answer). Accepting is the preferred,
  unmarked response; declining is dispreferred and usually delayed and
  explained.
  *Gives us:* a decline with an account ("I'd love to, but the oven's on")
  reads differently from a bare refusal, and both differ from silence.

## When it lands wrong

- **Austin, *How to Do Things with Words* (1962).** An act can fail in
  different ways: a misfire, where the conventional procedure is not
  properly carried out and the act does not come off, or an abuse, where it
  comes off but insincerely. An act also needs uptake, the hearer grasping
  what was done.
  ([summary](https://en.wikipedia.org/wiki/How_to_Do_Things_with_Words))
  *Gives us:* three distinct things to record per move: what was intended,
  what was actually done, and what the hearer took up. A flirt that comes
  out as an insult and a flirt heard as an insult are different failures.
- **Garfinkel, breaching experiments (*Studies in Ethnomethodology*,
  1967).** Students deliberately broke everyday conventions, such as acting
  as lodgers in their own homes, to expose the rules by the reactions:
  bewilderment, attempts to normalise ("are you ill?"), then anger.
  *Gives us:* the shape of a response to a breach. First try to make sense
  of it, then repair or reinterpret it, then sanction it. That sequence is
  the default out-of-frame handler.

## Games that built on this

- **Prom Week and its engine Comme il Faut (McCoy, Treanor, Samuel, Mateas,
  Wardrip-Fruin, UC Santa Cruz).** Built around the social exchange: a
  social action one character takes toward another, intended to change the
  relationship, together with how the other responds. The responder accepts
  or rejects. The game had over forty exchanges, and characters' choices
  were weighed by more than 5,000 "social considerations".
  ([developer blog](https://promweek.soe.ucsc.edu/?p=295))
  *Gives us:* the initiator/responder split with accept or reject, and a
  warning about scale.
- **Lesson from both.** A survey of emergent narrative notes that Versu and
  Prom Week both found authoring very hard.
  ([Tension Space Analysis, 2020](https://arxiv.org/pdf/2004.10808)) Prom
  Week needed thousands of considerations. Our counterweight is the pattern
  this repo already uses: shared defaults that every character inherits,
  and small per-character programs that override only what is theirs.

## What this adds up to

1. A frame is a shared convention and a person's stance within it is
   personal (Goffman, Versu).
2. The frame set should cover phatic contact, scripted transactions, and
   Fiske's four relational models.
3. Moves are a small general vocabulary (Searle plus Bales) whose
   conventional meaning depends on the frame, and whose face cost depends on
   the relationship (Brown and Levinson).
4. Opening or shifting a frame is a bid, answered by turning toward, away or
   against. Declines with and without an account differ (Gottman,
   conversation analysis).
5. Each move has an intent, a performance and an uptake that can each come
   apart (Austin).
6. Stepping outside the frame is a distinct event with a characteristic
   response: make sense of it, repair it, sanction it (Goffman, Garfinkel).
