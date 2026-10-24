# Interiority: what psychology offers

2026-10-24. Companion to `conversation-frames.md`, for the inside of a
villager: how they see themselves, how they model and misread others, how
feelings arise, what personality is, and how closeness forms. Each entry ends
with what it gives the design (`docs/design/conversation.md`). Findings that
later work only partly supports are marked.

## Personality as if-then programs

- **Mischel and Shoda, the cognitive-affective processing system (CAPS,
  1995).** Reconciles stable personality with behaviour that varies across
  situations. People differ in which mental units are active (how they
  construe situations, their expectations and beliefs, feelings, goals and
  values, competencies and self-regulation) and in how those connect to
  features of a situation. The stable thing is a behavioural signature: "if
  A, she does X; if B, she does Y". The signatures were first documented by
  observing children at a summer camp. Reviews note CAPS is a framework,
  not a set of content predictions.
  ([Wikipedia](https://en.wikipedia.org/wiki/Cognitive-affective_personality_system),
  [paper](https://psychology.columbia.edu/sites/default/files/2016-11/246.pdf))
  *Gives us:* the strongest support for a program per character. Ian's
  introvert who is anxious only with people she isn't easy with is an
  if-then signature. Traits such as the Big Five are useful shorthand for
  defaults; the program is the personality.
- **Fleeson, whole trait theory (2001).** Within a person, states such as
  how extraverted they act vary widely from moment to moment; a trait is the
  centre and spread of that distribution.
  *Gives us:* a roll from the person's own stream around their usual stance
  is psychologically realistic, not just noise.

## The self

- **Self-schemas (Markus, 1977) and possible selves (Markus and Nurius,
  1986).** People organise what they know about themselves around a few
  central self-descriptions, and they carry images of who they might become
  or fear becoming.
  *Gives us:* a self-concept is a few beliefs the program cares about
  ("I'm someone people can rely on"), plus hoped-for and feared selves that
  can drive goals.
- **Self-discrepancy theory (Higgins, 1987).** Actual, ideal and ought selves.
  A gap between actual and ideal is linked to dejection; between actual and
  ought, to agitation. Later studies give mixed support for the specific
  emotion mapping.
  ([paper](https://www.columbia.edu/cu/psychology/higgins/papers/HIGGINS%3DPSYCH%20REVIEW%201987.pdf))
  *Gives us:* a way for events to hurt differently: failing a hope saddens,
  failing a duty agitates. Use as a default, not a law.
- **Self-verification (Swann).** People seek and prefer feedback that confirms
  how they already see themselves, even when that view is negative.
  *Gives us:* someone with a poor self-view may distrust praise. A villager
  can be wrong about themselves and stay wrong.
- **Sociometer theory (Leary, 1995).** Self-esteem tracks how much a person
  feels valued and included by others.
  *Gives us:* bids turned away and conversations that go cold feed back
  into how a villager feels about themselves.

## Other people

- **Attribution (Heider 1958; Kelley 1967; Ross 1977; Jones and Nisbett
  1971).** People explain behaviour by the person or by the situation. They
  over-attribute others' behaviour to character (the fundamental attribution
  error), and their own to circumstances (actor-observer asymmetry).
  *Gives us:* the default way to read a move. Hal reads Mara's deflection as
  coldness (her character) rather than discretion (the queue), while Mara
  explains it by the situation. This is the misunderstanding engine.
- **Metaperception (Kenny and DePaulo, 1993).** People are only partly
  accurate about how others see them. They know their general reputation
  better than how a particular person sees them, and their beliefs about
  others' views track their own self-view closely.
  ([paper](https://www.simine.com/407/readings/Kenny_and_DePaulo_1993.pdf))
  *Gives us:* "what does Mara think of me?" defaults to a projection of the
  villager's own self-view, nudged by what Mara actually did. That makes
  second-order mistakes routine, as in life.
- **Rejection sensitivity (Downey and Feldman, 1996).** Some people anxiously
  expect rejection, readily perceive intentional rejection in ambiguous
  behaviour, and overreact. It is relatively stable and is linked to earlier
  painful rejection.
  ([paper](https://www.ffri.hr/~ibrdar/komunikacija/seminari/Downey,%201996%20-%20Rejection%20Sensitivity%20for%20intimate%20relationsh.pdf))
  *Gives us:* a ready parameter for how a villager takes up an answer to
  their bid. A deflection with an account reads as rejection to them anyway.
- **Attachment styles (Bowlby; Hazan and Shaver, 1987).** Secure, anxious and
  avoidant patterns in close relationships.
  *Gives us:* defaults for how much someone bids, how they read distance,
  and whether they pull back or press when turned away.

## Feelings

- **Appraisal theories (Lazarus 1991; Ortony, Clore and Collins, the OCC
  model, 1988).** Emotions come from how a person construes a situation.
  OCC sorts them into reactions to events (judged against goals), to agents'
  actions (judged against standards) and to objects (judged against tastes).
  Its 22 types and stated eliciting conditions made it a common basis for
  computational models; it doesn't cover everything (jealousy, envy) but
  extends by adding conditions.
  ([survey of game uses](https://arxiv.org/pdf/2307.10031))
  *Gives us:* the default `take_up`: a move is judged against the hearer's
  goals and standards and their attitude to the speaker. Gratitude and anger
  come out of the same machinery.
- **Mood and core affect (Russell 1980; Forgas, affect infusion 1995).**
  Moods are longer and vaguer than emotions, and they colour how ambiguous
  things are read.
  *Gives us:* a bad morning makes the same tease land worse. Mood is an
  input to `take_up`, not just an output.
- **Emotional contagion (Hatfield, Cacioppo and Rapson, 1993).** People pick
  up the emotions of those around them.
  *Gives us:* a cheerful tavern lifts the newcomer, a tense one spreads.

## How closeness forms

- **Interpersonal process model of intimacy (Reis and Shaver, 1988).**
  Intimacy grows when one person discloses and the listener responds, and
  what matters most is whether the discloser perceives the response as
  understanding and validating. Diary and friendship studies support
  perceived responsiveness as the link.
  ([Laurenceau et al. 1998](https://nmr.mgh.harvard.edu/publications/journal_articles/2791))
  *Gives us:* how a conversation becomes warm, in mechanism: disclosure,
  then the other's response, then the discloser's reading of that response.
  It fits the intent, performance and uptake split exactly.
- **Social penetration (Altman and Taylor, 1973) and reciprocal disclosure
  (Aron et al., 1997).** Relationships deepen by gradually widening and
  deepening what is shared, and matched, escalating disclosure produces
  closeness quickly.
  *Gives us:* confiding has to be earned. Bidding into it too early is a
  near-breach, and matching a disclosure is the warm answer.
- **Propinquity and familiarity (Festinger, Schachter and Back, 1950;
  Zajonc, mere exposure, 1968).** Physical nearness and repeated contact
  predict friendship.
  *Gives us:* in a village, neighbours and regulars grow familiar just by
  crossing paths. Greetings and small talk accumulate into something.
- **Interpersonal circumplex (Leary 1957; Wiggins; Kiesler).** Interpersonal
  behaviour varies on warmth and dominance. Responses tend to complement:
  warmth invites warmth, dominance invites deference.
  *Gives us:* the default way stances meet, which personal programs bend.

## Memory and judgement

- **Peak-end rule (Kahneman et al., 1993) and negativity bias (Baumeister et
  al., 2001, "Bad is stronger than good").** Experiences are remembered by
  their most intense moment and how they ended, and bad events weigh more
  than good ones of the same size.
  *Gives us:* the default `recall`: an evening is judged by its sharpest
  moment and its ending, and one cold exchange can outweigh several warm
  ones unless the person's program forgives.
- **Investment model (Rusbult, 1980).** Commitment rises with satisfaction
  and investment and falls with attractive alternatives.
  *Gives us:* later, when goals exist, a principled reason someone stays in
  a sour friendship, marriage or job.
- **Gossip (Dunbar, 1996; later work on its functions).** Gossip maintains
  bonds, spreads reputational information, and enforces norms.
  *Gives us:* a reason for catching up to carry third-party facts, and the
  route by which breaches become known.

## What this adds up to

1. Personality is a program of if-then signatures (CAPS); traits are
   shorthand for defaults; behaviour varies around a centre.
2. A self-concept is a few central beliefs plus hoped-for and feared selves;
   it can be wrong and resists correction.
3. People explain others by character and themselves by circumstance; that
   default, plus rejection sensitivity and mood, produces misreadings.
4. What a villager thinks others think of them is mostly their own
   self-view, projected.
5. Feelings come from appraising what happened against goals, standards and
   attitudes.
6. Warmth is made in conversation by disclosure answered by responsiveness,
   as the discloser perceives it; it deepens gradually and must be matched.
7. Memory keeps the peak and the end, and the bad weighs more.
