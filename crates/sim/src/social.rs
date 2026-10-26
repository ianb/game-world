//! The inner side of conversations: what each person carries into a talk, how they choose
//! what to say, how it comes out, how they take what the other says, and what they keep
//! afterwards. Background: `docs/research/conversation-frames.md` and
//! `docs/research/interiority-psychology.md`; design: `docs/design/conversation.md`.
//!
//! Every hook on `Interlocutor` has a default here, built from the research. A character
//! overrides the hooks that make them who they are, the way `toolbox::human` works for
//! needs. Nothing here stores how two people feel about each other: each person keeps
//! episodes, and `Memory::regard` reads a view of someone from them when asked.
use crate::brain::Ctx;
use crate::ids::VillagerId;
use crate::protocol::{Act, Attribution, BidAnswer, Caught, Frame, Gesture, Stance, Topic};
use crate::time::Tick;
use rand::RngExt;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

impl Frame {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Frame::Greeting => "greeting",
            Frame::Transaction => "transaction",
            Frame::SmallTalk => "small talk",
            Frame::Confiding => "confiding",
        }
    }

    /// Whether a move belongs in this frame. Moves nobody means (a jab, an awkward moment)
    /// are failures inside the frame, not breaches of it.
    #[must_use]
    pub fn allows(self, kind: Gesture) -> bool {
        use Gesture as M;
        if matches!(kind, M::Leave | M::Jab | M::Awkward | M::Rebuff | M::Pry) {
            return true;
        }
        match self {
            Frame::Greeting => matches!(
                kind,
                M::Ask | M::Tell | M::Agree | M::Joke | M::Thank | M::Compliment
            ),
            Frame::Transaction => matches!(
                kind,
                M::Request
                    | M::Offer
                    | M::Thank
                    | M::Ask
                    | M::Tell
                    | M::Agree
                    | M::Decline
                    | M::Joke
                    | M::Compliment
            ),
            Frame::SmallTalk => matches!(
                kind,
                M::Ask
                    | M::Tell
                    | M::Agree
                    | M::Joke
                    | M::Tease
                    | M::Compliment
                    | M::Complain
                    | M::Thank
                    | M::Offer
                    | M::Request
                    | M::Decline
            ),
            Frame::Confiding => matches!(
                kind,
                M::Disclose
                    | M::Sympathise
                    | M::Ask
                    | M::Agree
                    | M::Tell
                    | M::Thank
                    | M::Complain
                    | M::Offer
            ),
        }
    }

    /// Moves that are this frame's script, which leave little room to be misread: the
    /// request and the offer at a counter, the how-are-you of a greeting.
    #[must_use]
    pub fn scripted(self, kind: Gesture) -> bool {
        use Gesture as M;
        match self {
            Frame::Transaction => matches!(kind, M::Request | M::Offer | M::Thank),
            Frame::Greeting => matches!(kind, M::Ask | M::Tell | M::Thank),
            Frame::SmallTalk | Frame::Confiding => false,
        }
    }

    /// Frames you can bid into from this one without stepping outside convention.
    #[must_use]
    pub fn bids_to(self, to: Frame) -> bool {
        matches!(
            (self, to),
            (
                Frame::Greeting | Frame::Transaction | Frame::Confiding,
                Frame::SmallTalk
            ) | (Frame::SmallTalk, Frame::Confiding)
        )
    }
}

impl Gesture {
    /// Every gesture, in declaration order. `listed` below has to name each one, so a new
    /// gesture fails to compile until it is added here too.
    pub const ALL: [Gesture; 19] = [
        Gesture::Ask,
        Gesture::Tell,
        Gesture::Agree,
        Gesture::Joke,
        Gesture::Tease,
        Gesture::Compliment,
        Gesture::Complain,
        Gesture::Request,
        Gesture::Offer,
        Gesture::Thank,
        Gesture::Sympathise,
        Gesture::Disclose,
        Gesture::Decline,
        Gesture::Invite,
        Gesture::Leave,
        Gesture::Jab,
        Gesture::Awkward,
        Gesture::Rebuff,
        Gesture::Pry,
    ];

    #[deny(clippy::wildcard_enum_match_arm)]
    #[cfg(test)]
    fn listed(self) -> bool {
        use Gesture as G;
        match self {
            G::Ask
            | G::Tell
            | G::Agree
            | G::Joke
            | G::Tease
            | G::Compliment
            | G::Complain
            | G::Request
            | G::Offer
            | G::Thank
            | G::Sympathise
            | G::Disclose
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Jab
            | G::Awkward
            | G::Rebuff
            | G::Pry => Gesture::ALL.contains(&self),
        }
    }

    /// How warm the move is, from -3 to 2, as conventionally meant.
    #[must_use]
    pub fn warmth(self) -> i8 {
        use Gesture as M;
        match self {
            M::Compliment | M::Sympathise => 2,
            M::Agree | M::Joke | M::Tease | M::Offer | M::Thank | M::Disclose | M::Invite => 1,
            M::Ask | M::Tell | M::Request | M::Leave => 0,
            M::Complain | M::Decline | M::Awkward | M::Pry => -1,
            M::Jab => -2,
            M::Rebuff => -3,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        use Gesture as M;
        match self {
            M::Ask => "ask",
            M::Tell => "tell",
            M::Agree => "agree",
            M::Joke => "joke",
            M::Tease => "tease",
            M::Compliment => "compliment",
            M::Complain => "complain",
            M::Request => "request",
            M::Offer => "offer",
            M::Thank => "thank",
            M::Sympathise => "sympathise",
            M::Disclose => "confide",
            M::Decline => "decline",
            M::Invite => "invite",
            M::Leave => "leave",
            M::Jab => "a dig",
            M::Awkward => "something awkward",
            M::Rebuff => "a rebuff",
            M::Pry => "prying",
        }
    }

    /// The move a suspicious ear makes of this one.
    #[must_use]
    pub fn worse(self) -> Gesture {
        use Gesture as M;
        match self {
            M::Tease | M::Joke | M::Complain => M::Jab,
            M::Ask | M::Invite | M::Request => M::Pry,
            M::Decline => M::Rebuff,
            M::Compliment | M::Sympathise | M::Agree => M::Awkward,
            M::Disclose => M::Complain,
            other => other,
        }
    }

    /// The move a generous ear makes of this one.
    #[must_use]
    pub fn better(self) -> Gesture {
        use Gesture as M;
        match self {
            M::Jab => M::Tease,
            M::Rebuff => M::Decline,
            M::Decline => M::Agree,
            M::Awkward => M::Joke,
            M::Pry => M::Ask,
            M::Complain => M::Disclose,
            other => other,
        }
    }

    /// Moves that are easy to get wrong.
    #[must_use]
    pub fn hard(self) -> bool {
        matches!(self, Gesture::Tease | Gesture::Joke | Gesture::Compliment)
    }
}

/// How far a turn carries: confiding is said low, a laugh carries, a dig or a complaint is
/// said loud.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Carry {
    Low,
    Ordinary,
    Carrying,
    Loud,
}

impl Carry {
    /// Tiles within which a listener catches the words, the gist, and raised voices.
    #[must_use]
    pub fn reach(self) -> (u16, u16, u16) {
        match self {
            Carry::Low => (2, 3, 0),
            Carry::Ordinary => (4, 8, 0),
            Carry::Carrying => (5, 10, 16),
            Carry::Loud => (6, 12, 20),
        }
    }

    /// What a listener `distance` tiles from the speaker catches, if anything.
    #[must_use]
    pub fn caught_at(self, distance: u16) -> Option<Caught> {
        let (words, gist, noise) = self.reach();
        if distance <= words {
            Some(Caught::Close)
        } else if distance <= gist {
            Some(Caught::Gist)
        } else if distance <= noise {
            Some(Caught::Noise)
        } else {
            None
        }
    }
}

impl Gesture {
    /// How far this gesture carries. Every gesture is listed.
    #[deny(clippy::wildcard_enum_match_arm)]
    #[must_use]
    pub fn carry(self) -> Carry {
        use Gesture as G;
        match self {
            G::Disclose | G::Sympathise => Carry::Low,
            G::Ask
            | G::Tell
            | G::Agree
            | G::Tease
            | G::Compliment
            | G::Request
            | G::Offer
            | G::Thank
            | G::Decline
            | G::Invite
            | G::Leave
            | G::Awkward
            | G::Pry => Carry::Ordinary,
            G::Joke => Carry::Carrying,
            G::Complain | G::Jab | G::Rebuff => Carry::Loud,
        }
    }
}

/// How far an act carries: a bid to confide is made low, like the confiding itself.
#[must_use]
pub fn act_carry(act: &Act) -> Carry {
    match act {
        Act::Bid {
            frame: Frame::Confiding,
            ..
        } => Carry::Low,
        other => act_gesture(other).carry(),
    }
}

/// Reads an act as the gesture it amounts to.
#[must_use]
pub fn act_gesture(act: &Act) -> Gesture {
    match act {
        Act::Gesture { gesture, .. } => *gesture,
        Act::Bid { .. } => Gesture::Invite,
        Act::Answer { answer } => match answer {
            BidAnswer::Toward => Gesture::Agree,
            BidAnswer::Away { .. } => Gesture::Decline,
            BidAnswer::Against => Gesture::Rebuff,
        },
    }
}

/// A person's settled tendencies, the parameters the shared defaults read. Not the
/// personality: that is the character's program (Mischel and Shoda's if-then signatures).
/// Scales are small integers; 0 is ordinary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Persona {
    /// How much they seek talk, -3 (avoids it) to 3.
    pub sociable: i8,
    /// How warm they come across by default, -3 to 3.
    pub warmth: i8,
    /// How well what they mean comes out, 0 to 10.
    pub skill: u8,
    /// How readily they hear rejection in ambiguous answers, 0 to 10 (Downey and Feldman).
    pub rejection_sensitivity: u8,
    /// How readily they excuse others by circumstance rather than character, -3 to 3.
    pub charity: i8,
    /// How readily they keep what they know about others to themselves, 0 to 10. At 2 or
    /// less they pass on even what was told them in confidence.
    pub discretion: u8,
}

impl Default for Persona {
    fn default() -> Persona {
        Persona {
            sociable: 0,
            warmth: 0,
            skill: 5,
            rejection_sensitivity: 3,
            charity: 0,
            discretion: 5,
        }
    }
}

/// A remembered conversation, as this person experienced it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub tick: Tick,
    pub with: VillagerId,
    pub frame: Frame,
    /// The most intense moment and the last one, -3 to 3.
    pub peak: i8,
    pub end: i8,
    /// How the other person seemed, from their moves as this person read them.
    pub they_seemed: i8,
    pub notes: Vec<String>,
}

/// A turn someone perceived: who was talking, to whom, about what, and how (the gesture,
/// as they took it). Too far away for the words, the topic is missing; too far for more
/// than raised voices, so is the listener. Character code reads these through
/// `Memory::heard`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heard {
    pub tick: Tick,
    pub speaker: VillagerId,
    /// Who it was said to, when they could tell.
    pub to: Option<VillagerId>,
    pub gesture: Gesture,
    pub topic: Option<Topic>,
    /// Their own name was in it, which carries further than the words.
    pub about_me: bool,
    pub caught: Caught,
}

/// What a person keeps. Old things fall away; a character can override what to keep.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub episodes: Vec<Episode>,
    pub heard: Vec<Heard>,
}

const MAX_EPISODES: usize = 60;
const MAX_HEARD: usize = 150;
/// How many recent episodes with someone shape the view of them.
const REGARD_WINDOW: usize = 8;

/// A view of someone, read from memory when needed rather than stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Regard {
    /// How many conversations they remember having.
    pub familiarity: u16,
    /// How they feel about them, -3 to 3: peak and end of recent episodes, the bad
    /// counting double (Kahneman; Baumeister).
    pub feeling: i8,
    /// How the other seemed last time.
    pub last_seemed: i8,
}

/// `n / d` rounded half away from zero, for positive `d`.
fn rounded(n: i32, d: i32) -> i32 {
    (2 * n + if n >= 0 { d } else { -d }) / (2 * d)
}

impl Memory {
    pub fn remember(&mut self, e: Episode) {
        self.episodes.push(e);
        if self.episodes.len() > MAX_EPISODES {
            self.episodes.remove(0);
        }
    }

    pub fn hear(&mut self, h: Heard) {
        self.heard.push(h);
        if self.heard.len() > MAX_HEARD {
            self.heard.remove(0);
        }
    }

    #[must_use]
    pub fn regard(&self, other: VillagerId) -> Regard {
        let mine: Vec<&Episode> = self.episodes.iter().filter(|e| e.with == other).collect();
        let Some(last) = mine.last() else {
            return Regard::default();
        };
        let recent = &mine[mine.len().saturating_sub(REGARD_WINDOW)..];
        // Mean of (peak + end) / 2, rounded, with bad episodes weighing double.
        let (mut sum, mut weight) = (0i32, 0i32);
        for e in recent {
            let v = i32::from(e.peak) + i32::from(e.end);
            let w = if v < 0 { 2 } else { 1 };
            sum += v * w;
            weight += w;
        }
        Regard {
            familiarity: u16::try_from(mine.len()).unwrap_or(u16::MAX),
            feeling: i8::try_from(rounded(sum, (weight * 2).max(1)).clamp(-3, 3)).unwrap_or(0),
            last_seemed: last.they_seemed,
        }
    }

    /// Whether they told `other` about `topic` in a conversation since `tick`.
    #[must_use]
    pub fn told_since(&self, other: VillagerId, topic: &Topic, tick: Tick) -> bool {
        let note = format!("told {topic}");
        self.episodes
            .iter()
            .any(|e| e.with == other && e.tick >= tick && e.notes.contains(&note))
    }

    /// A query over what they heard: chain filters, then `iter`, `any`, `count` or `last`.
    #[must_use]
    pub fn heard(&self) -> HeardQuery<'_> {
        HeardQuery {
            all: &self.heard,
            speaker: None,
            to: None,
            gesture: None,
            topic: None,
            about: None,
            since: Tick(0),
            words: false,
        }
    }
}

/// Filters over a person's `Heard` memories. Unset filters match everything.
#[derive(Clone, Debug)]
pub struct HeardQuery<'a> {
    all: &'a [Heard],
    speaker: Option<VillagerId>,
    to: Option<VillagerId>,
    gesture: Option<Gesture>,
    topic: Option<&'a str>,
    about: Option<VillagerId>,
    since: Tick,
    words: bool,
}

impl<'a> HeardQuery<'a> {
    #[must_use]
    pub fn from(mut self, speaker: VillagerId) -> Self {
        self.speaker = Some(speaker);
        self
    }
    #[must_use]
    pub fn to(mut self, listener: VillagerId) -> Self {
        self.to = Some(listener);
        self
    }
    #[must_use]
    pub fn gesture(mut self, gesture: Gesture) -> Self {
        self.gesture = Some(gesture);
        self
    }
    #[must_use]
    pub fn topic(mut self, key: &'a str) -> Self {
        self.topic = Some(key);
        self
    }
    /// About this person (the topic's `about`).
    #[must_use]
    pub fn about(mut self, who: VillagerId) -> Self {
        self.about = Some(who);
        self
    }
    #[must_use]
    pub fn since(mut self, tick: Tick) -> Self {
        self.since = tick;
        self
    }
    /// Only those close enough to catch the words.
    #[must_use]
    pub fn words(mut self) -> Self {
        self.words = true;
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a Heard> + '_ {
        self.all.iter().filter(move |h| {
            h.tick >= self.since
                && self.speaker.is_none_or(|v| h.speaker == v)
                && self.to.is_none_or(|v| h.to == Some(v))
                && self.gesture.is_none_or(|g| h.gesture == g)
                && self
                    .topic
                    .is_none_or(|k| h.topic.as_ref().is_some_and(|t| t.is(k)))
                && self
                    .about
                    .is_none_or(|v| h.topic.as_ref().is_some_and(|t| t.about == Some(v)))
                && (!self.words || matches!(h.caught, Caught::Addressed | Caught::Close))
        })
    }
    #[must_use]
    pub fn any(&self) -> bool {
        self.iter().next().is_some()
    }
    #[must_use]
    pub fn count(&self) -> usize {
        self.iter().count()
    }
    #[must_use]
    pub fn last(&self) -> Option<&'a Heard> {
        self.iter().last()
    }
}

/// One turn as the conversation records it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeatRecord {
    pub tick: Tick,
    pub who: VillagerId,
    pub meant: Act,
    pub did: Act,
    pub breach: bool,
}

/// How one participant has felt so far.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Felt {
    pub peak: i8,
    pub last: i8,
    pub seemed: i16,
    pub notes: Vec<String>,
}

impl Felt {
    /// Takes in one reading: the most intense moment wins, ties go to the bad. Only
    /// `telling` moves (not a frame's script) count toward how the other seemed.
    pub fn feel(&mut self, warmth: i8, telling: bool) {
        if warmth.abs() > self.peak.abs() || (warmth.abs() == self.peak.abs() && warmth < 0) {
            self.peak = warmth;
        }
        self.last = warmth;
        if telling {
            self.seemed += i16::from(warmth);
        }
    }
}

/// A conversation in progress. `a` opened it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: u32,
    pub a: VillagerId,
    pub b: VillagerId,
    pub frame: Frame,
    /// What it is about now; a bid that is taken changes it.
    pub topic: Topic,
    /// What it was opened about, which is what `TalkEnded` reports.
    pub booked: Topic,
    pub started: Tick,
    pub ends: Tick,
    /// Whose turn it is.
    pub next: VillagerId,
    /// Turns taken by `a` and `b` in the current frame.
    pub turns: [u16; 2],
    pub beats: Vec<BeatRecord>,
    /// A bid waiting for an answer: who must answer, into which frame, about what.
    pub bid: Option<(VillagerId, Frame, Topic)>,
    /// Who has to respond to the other stepping outside the frame.
    pub breach_reply: Option<VillagerId>,
    /// How `a` and `b` have felt so far.
    pub felt: [Felt; 2],
}

impl Conversation {
    #[must_use]
    pub fn other(&self, who: VillagerId) -> VillagerId {
        if who == self.a { self.b } else { self.a }
    }

    #[must_use]
    pub fn side(&self, who: VillagerId) -> usize {
        usize::from(who != self.a)
    }
}

/// What a hook can see: the town, the conversation so far, and who else is in earshot.
#[derive(Debug)]
pub struct Situation<'a> {
    pub ctx: &'a Ctx<'a>,
    pub me: VillagerId,
    /// The other participant (for a bystander, the speaker).
    pub other: VillagerId,
    pub frame: Frame,
    pub topic: &'a Topic,
    pub beats: &'a [BeatRecord],
    pub bystanders: &'a [VillagerId],
    /// Whether `me` opened the conversation.
    pub initiator: bool,
    /// Turns `me` has taken in the current frame.
    pub turn: u16,
}

impl Situation<'_> {
    /// The other person's last act, if any.
    #[must_use]
    pub fn last_from_other(&self) -> Option<&Act> {
        self.beats
            .iter()
            .rev()
            .find(|b| b.who == self.other)
            .map(|b| &b.did)
    }

    /// Whether `me` has already made `gesture` in this conversation.
    #[must_use]
    pub fn i_did(&self, gesture: Gesture) -> bool {
        self.beats
            .iter()
            .any(|b| b.who == self.me && act_gesture(&b.did) == gesture)
    }

    /// Whether `me` has already bid into `frame` in this conversation.
    #[must_use]
    pub fn i_bid(&self, frame: Frame) -> bool {
        self.beats
            .iter()
            .any(|b| b.who == self.me && matches!(b.did, Act::Bid { frame: f, .. } if f == frame))
    }

    #[must_use]
    pub fn private(&self) -> bool {
        self.bystanders.is_empty()
    }
}

/// How a hearer took a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uptake {
    pub taken_as: Gesture,
    pub warmth: i8,
    pub attribution: Attribution,
}

/// Mood from the body: pressing needs make anyone shorter.
#[must_use]
pub fn needs_mood(ctx: &Ctx<'_>, me: VillagerId) -> i8 {
    let n = &ctx.villager(me).needs;
    let worst = n.hunger.max(n.thirst).max(n.bladder).max(n.sleep);
    if worst >= crate::needs::URGENT {
        -2
    } else if worst >= crate::needs::PRESSING {
        -1
    } else {
        0
    }
}

/// A percentile roll, 0 to 99.
pub fn roll(rng: &mut Pcg32) -> i16 {
    i16::try_from(rng.random_range(0..100u32)).unwrap_or(0)
}

/// A gesture about `topic`.
#[must_use]
pub fn act(gesture: Gesture, topic: &Topic) -> Act {
    Act::Gesture {
        gesture,
        topic: topic.clone(),
    }
}

/// Picks one of `options` with integer weights; zero weights never win, and if every weight
/// is zero the last option is taken.
fn weighted(rng: &mut Pcg32, options: &[(Act, i16)]) -> Act {
    let total: i16 = options.iter().map(|(_, w)| (*w).max(0)).sum();
    if total <= 0 {
        return options[options.len() - 1].0.clone();
    }
    let mut pick =
        i16::try_from(rng.random_range(0..u32::try_from(total).unwrap_or(1))).unwrap_or(0);
    for (act, w) in options {
        let w = (*w).max(0);
        if pick < w {
            return act.clone();
        }
        pick -= w;
    }
    options[options.len() - 1].0.clone()
}

/// A conversation partner. Every method but the three accessors has a default from the
/// research; a character overrides the ones that make them who they are.
pub trait Interlocutor {
    fn persona(&self) -> Persona;
    fn memory(&self) -> &Memory;
    fn memory_mut(&mut self) -> &mut Memory;

    /// How they feel in general right now, -3 to 3.
    fn mood(&self, ctx: &Ctx<'_>, me: VillagerId) -> i8 {
        needs_mood(ctx, me)
    }

    /// What each gesture means to them, -3 to 3. Required, and written as a match with no
    /// catch-all arm (`#[deny(clippy::wildcard_enum_match_arm)]`), so a new gesture makes
    /// every character decide. `Gesture::warmth` is the shared starting point.
    fn warmth_of(&self, gesture: Gesture) -> i8;

    /// Something on their mind they might want to confide.
    fn worry(&self, ctx: &Ctx<'_>, me: VillagerId) -> Option<Topic> {
        let _ = (ctx, me);
        None
    }

    /// How they come into this turn.
    fn stance(&self, s: &Situation<'_>) -> Stance {
        default_stance(self, s)
    }

    /// What they mean to do next.
    fn next_act(&mut self, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
        default_next_act(self, s, rng)
    }

    /// How they answer a bid into `frame`.
    fn answer_bid(&mut self, s: &Situation<'_>, frame: Frame, rng: &mut Pcg32) -> BidAnswer {
        default_answer_bid(self, s, frame, rng)
    }

    /// What actually comes out when they try to do `meant`.
    fn perform(&mut self, s: &Situation<'_>, meant: &Act, stance: Stance, rng: &mut Pcg32) -> Act {
        default_perform(self, s, meant, stance, rng)
    }

    /// How they take what `speaker` did.
    fn take_up(
        &mut self,
        s: &Situation<'_>,
        speaker: VillagerId,
        did: &Act,
        rng: &mut Pcg32,
    ) -> Uptake {
        default_take_up(self, s, speaker, did, rng)
    }

    /// What they do when the other steps outside the frame.
    fn out_of_frame(&mut self, s: &Situation<'_>, did: &Act, rng: &mut Pcg32) -> Act {
        default_out_of_frame(self, s, did, rng)
    }

    fn remember(&mut self, e: Episode) {
        self.memory_mut().remember(e);
    }

    fn hear(&mut self, h: Heard) {
        self.memory_mut().hear(h);
    }
}

/// Footing: cold with people they think badly of, guarded when out of sorts or with
/// strangers if they are not sociable, otherwise open.
pub fn default_stance<I: Interlocutor + ?Sized>(me: &I, s: &Situation<'_>) -> Stance {
    let r = me.memory().regard(s.other);
    let p = me.persona();
    if r.feeling <= -2 {
        Stance::Cold
    } else if me.mood(s.ctx, s.me) <= -2 || (p.sociable < 0 && r.familiarity < 3) {
        Stance::Guarded
    } else {
        Stance::Open
    }
}

/// Whether they want the conversation to go on past its business.
fn wants_more<I: Interlocutor + ?Sized>(me: &I, s: &Situation<'_>, rng: &mut Pcg32) -> bool {
    let p = me.persona();
    let r = me.memory().regard(s.other);
    let mut chance = 20 + 10 * i16::from(p.sociable) + 10 * i16::from(r.feeling.max(0));
    if r.familiarity >= 3 {
        chance += 10;
    }
    if me.mood(s.ctx, s.me) < 0 {
        chance -= 15;
    }
    if me.worry(s.ctx, s.me).is_some() && r.familiarity >= 2 {
        chance += 30;
    }
    roll(rng) < chance
}

/// The default turn: the frame's business first, then talk shaped by stance, mood and
/// history, with a bid to go deeper when something is on their mind.
pub fn default_next_act<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    rng: &mut Pcg32,
) -> Act {
    let topic = s.topic;
    match s.frame {
        Frame::Transaction => match (s.initiator, s.turn) {
            (true, 0) => act(Gesture::Request, topic),
            // Thanks for a gift; otherwise serve what was asked for.
            (false, 0) if s.last_from_other().map(act_gesture) == Some(Gesture::Offer) => {
                act(Gesture::Thank, topic)
            }
            (false, 0) => act(Gesture::Offer, topic),
            (_, 1) => act(Gesture::Thank, topic),
            _ if wants_more(me, s, rng) => Act::Bid {
                frame: Frame::SmallTalk,
                topic: s.ctx.topic("day"),
            },
            _ => act(Gesture::Leave, topic),
        },
        Frame::Greeting => match s.turn {
            0 => act(Gesture::Ask, topic),
            1 => match default_gossip(me, s, rng) {
                Some(news) => act(Gesture::Tell, &news),
                None => act(Gesture::Tell, topic),
            },
            _ if wants_more(me, s, rng) => Act::Bid {
                frame: Frame::SmallTalk,
                topic: s.ctx.topic("day"),
            },
            _ => act(Gesture::Leave, topic),
        },
        Frame::SmallTalk => small_talk_act(me, s, rng),
        Frame::Confiding => confiding_act(me, s),
    }
}

/// How long something heard stays worth passing on.
const NEWS_DAYS: u32 = 7;

/// Something they know about a third person that this listener has not had from them:
/// heard in words in the last week, not about the listener or themselves, and not
/// something they have heard the listener talk about. A confidence (disclosed to them
/// directly), or news between two people told to the second of them, is passed on only at
/// discretion 2 or less. Whether they pass it on at all is a roll weighted by discretion.
pub fn default_gossip<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    rng: &mut Pcg32,
) -> Option<Topic> {
    let p = me.persona();
    let since = s.ctx.tick.minus(NEWS_DAYS * crate::time::TICKS_PER_DAY);
    let news = me
        .memory()
        .heard()
        .words()
        .since(since)
        .iter()
        .filter(|h| h.speaker != s.other)
        .filter(|h| {
            let confidence = h.gesture == Gesture::Disclose && h.caught == Caught::Addressed;
            !confidence || p.discretion <= 2
        })
        .filter_map(|h| h.topic.as_ref())
        .filter(|t| t.about.is_some_and(|a| a != s.me && a != s.other))
        // For news between two people, only the indiscreet tell the second one.
        .filter(|t| t.with != Some(s.me) && (t.with != Some(s.other) || p.discretion <= 2))
        .filter(|t| !me.memory().told_since(s.other, t, since))
        // Not to someone they have heard talking about it: they know already.
        .filter(|t| {
            !me.memory()
                .heard()
                .from(s.other)
                .topic(&t.key)
                .since(since)
                .iter()
                .any(|h| {
                    h.topic
                        .as_ref()
                        .is_some_and(|ht| ht.about == t.about && ht.with == t.with)
                })
        })
        .last()?
        .clone();
    (roll(rng) < 10 * (10 - i16::from(p.discretion.min(10)))).then_some(news)
}

/// Days before someone tells the same person the same worry again.
const RETELL_DAYS: u32 = 7;

/// Small talk: a bid to go deeper when something is on their mind, to someone they like
/// who has not heard it this week; leaving early when cold or shy; otherwise talk weighted
/// by footing and history.
fn small_talk_act<I: Interlocutor + ?Sized>(me: &I, s: &Situation<'_>, rng: &mut Pcg32) -> Act {
    let p = me.persona();
    let r = me.memory().regard(s.other);
    let mood = me.mood(s.ctx, s.me);
    let stance = me.stance(s);
    let topic = s.topic;
    if s.turn >= 1
        && let Some(news) = default_gossip(me, s, rng)
    {
        return act(Gesture::Tell, &news);
    }
    if let Some(worry) = me.worry(s.ctx, s.me)
        && s.turn >= 1
        && r.familiarity >= 2
        && r.feeling >= 1
        && !s.i_bid(Frame::Confiding)
        && !me.memory().told_since(
            s.other,
            &worry,
            s.ctx.tick.minus(RETELL_DAYS * crate::time::TICKS_PER_DAY),
        )
        && roll(rng) < 60
    {
        return Act::Bid {
            frame: Frame::Confiding,
            topic: worry,
        };
    }
    if stance == Stance::Cold && s.turn >= 1 {
        return act(Gesture::Leave, topic);
    }
    if s.turn >= 5 || (p.sociable < 0 && s.turn >= 2 && r.familiarity < 3) {
        return act(Gesture::Leave, topic);
    }
    let open = stance == Stance::Open;
    let other_warm = s
        .last_from_other()
        .is_some_and(|a| me.warmth_of(act_gesture(a)) > 0);
    weighted(
        rng,
        &[
            (act(Gesture::Tell, topic), 3),
            (act(Gesture::Ask, topic), 2),
            (act(Gesture::Agree, topic), if other_warm { 3 } else { 0 }),
            (
                act(Gesture::Joke, topic),
                if open {
                    1 + i16::from(p.warmth.max(0))
                } else {
                    0
                },
            ),
            (
                act(Gesture::Tease, topic),
                if open && r.familiarity >= 4 && r.feeling >= 1 {
                    2
                } else {
                    0
                },
            ),
            (
                act(Gesture::Compliment, topic),
                i16::from(p.warmth >= 1 && r.feeling >= 0),
            ),
            (
                act(Gesture::Complain, topic),
                if mood < 0 || stance == Stance::Cold {
                    3
                } else {
                    0
                },
            ),
        ],
    )
}

/// Confiding: say what is on their mind, or answer a disclosure with sympathy if warm.
fn confiding_act<I: Interlocutor + ?Sized>(me: &I, s: &Situation<'_>) -> Act {
    let p = me.persona();
    let stance = me.stance(s);
    let topic = s.topic;
    if let Some(worry) = me.worry(s.ctx, s.me)
        && !s.i_did(Gesture::Disclose)
    {
        return act(Gesture::Disclose, &worry);
    }
    if s.turn >= 3 {
        return act(Gesture::Leave, topic);
    }
    match s.last_from_other().map(act_gesture) {
        Some(Gesture::Disclose | Gesture::Complain) if stance != Stance::Cold && p.warmth >= 0 => {
            act(Gesture::Sympathise, topic)
        }
        Some(Gesture::Disclose) => act(Gesture::Ask, topic),
        _ => act(Gesture::Agree, topic),
    }
}

/// Toward when it fits how close they are; confiding needs history, warmth and privacy
/// (social penetration), and a decline in public comes with a reason.
pub fn default_answer_bid<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    frame: Frame,
    rng: &mut Pcg32,
) -> BidAnswer {
    let p = me.persona();
    let r = me.memory().regard(s.other);
    let mood = me.mood(s.ctx, s.me);
    let _ = rng;
    match frame {
        Frame::Confiding => {
            if r.feeling <= -2 {
                BidAnswer::Against
            } else if !s.private() {
                BidAnswer::Away { account: true }
            } else if r.familiarity >= 3 && r.feeling >= 0 && mood > -2 {
                BidAnswer::Toward
            } else {
                BidAnswer::Away {
                    account: p.warmth >= 0,
                }
            }
        }
        Frame::SmallTalk => {
            if r.feeling <= -2 {
                BidAnswer::Away { account: false }
            } else if mood <= -2 && p.sociable <= 0 {
                BidAnswer::Away { account: true }
            } else if p.sociable >= 0 || r.familiarity >= 2 {
                BidAnswer::Toward
            } else {
                BidAnswer::Away { account: true }
            }
        }
        _ => BidAnswer::Toward,
    }
}

/// Hard moves (teasing, joking, complimenting) misfire with low skill, a bad mood, or a
/// guarded or anxious footing; a reasoned refusal can come out curt.
pub fn default_perform<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    meant: &Act,
    stance: Stance,
    rng: &mut Pcg32,
) -> Act {
    let p = me.persona();
    let mut chance = (10 - i16::from(p.skill.min(10))) * 3;
    chance += match stance {
        Stance::Open => 0,
        Stance::Guarded => 10,
        Stance::Cold => 15,
        Stance::Anxious => 20,
    };
    if me.mood(s.ctx, s.me) < 0 {
        chance += 10;
    }
    match meant {
        Act::Gesture { gesture, topic } if gesture.hard() && roll(rng) < chance => {
            let came_out = match gesture {
                Gesture::Tease => Gesture::Jab,
                _ => Gesture::Awkward,
            };
            act(came_out, topic)
        }
        Act::Answer {
            answer: BidAnswer::Away { account: true },
        } if p.skill < 5 && roll(rng) < chance => Act::Answer {
            answer: BidAnswer::Away { account: false },
        },
        other => other.clone(),
    }
}

/// The biased roll: rejection sensitivity, a bad mood, a poor view of the speaker and a
/// curt answer push the reading worse; charity and a good view push it better. A frame's
/// scripted moves are rarely misread. A bad
/// reading is put down to character unless they are charitable (fundamental attribution
/// error), a little less when others were around to explain it.
pub fn default_take_up<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    speaker: VillagerId,
    did: &Act,
    rng: &mut Pcg32,
) -> Uptake {
    let p = me.persona();
    let r = me.memory().regard(speaker);
    let mood = me.mood(s.ctx, s.me);
    let base = act_gesture(did);
    let curt = matches!(
        did,
        Act::Answer {
            answer: BidAnswer::Away { account: false }
        }
    );
    let mut worse = 2 + i16::from(p.rejection_sensitivity) * 3;
    if mood < 0 {
        worse += 10;
    }
    if r.feeling < 0 {
        worse += 10 * i16::from(-r.feeling);
    }
    if curt {
        worse += 15;
    }
    worse -= 6 * i16::from(r.feeling.max(0));
    if s.frame.scripted(base) {
        worse /= 4;
    }
    let better = (5 + 4 * i16::from(p.charity) + 4 * i16::from(r.feeling.max(0))).clamp(0, 40);
    let worse = worse.clamp(0, 80);
    let x = roll(rng);
    let taken_as = if x < worse {
        base.worse()
    } else if x >= 100 - better {
        base.better()
    } else {
        base
    };
    let mut warmth = me.warmth_of(taken_as);
    // A reason given with a refusal softens it, if it is heard as the refusal it was.
    if taken_as == base
        && matches!(
            did,
            Act::Answer {
                answer: BidAnswer::Away { account: true }
            }
        )
    {
        warmth += 1;
    }
    if mood <= -2 {
        warmth -= 1;
    }
    let warmth = warmth.clamp(-3, 3);
    let attribution = if warmth >= 0 {
        Attribution::Character
    } else {
        let mut character = 70 - 12 * i16::from(p.charity);
        if !s.bystanders.is_empty() {
            character -= 15;
        }
        if roll(rng) < character.clamp(10, 95) {
            Attribution::Character
        } else {
            Attribution::Situation
        }
    };
    Uptake {
        taken_as,
        warmth,
        attribution,
    }
}

/// Garfinkel's sequence: with someone they like, repair it by laughing it off; with
/// someone they think badly of, sanction it; otherwise try to make sense of it.
pub fn default_out_of_frame<I: Interlocutor + ?Sized>(
    me: &I,
    s: &Situation<'_>,
    did: &Act,
    rng: &mut Pcg32,
) -> Act {
    let _ = (did, rng);
    let r = me.memory().regard(s.other);
    if r.feeling >= 1 || r.familiarity >= 6 {
        act(Gesture::Joke, s.topic)
    } else if r.feeling <= -1 {
        act(Gesture::Decline, s.topic)
    } else {
        act(Gesture::Ask, s.topic)
    }
}

/// Someone with no program of their own: default persona, no memory. Remembers nothing,
/// so nothing about them needs saving.
#[derive(Debug, Default)]
pub struct Plain {
    memory: Memory,
}

impl Interlocutor for Plain {
    fn persona(&self) -> Persona {
        Persona::default()
    }
    fn warmth_of(&self, gesture: Gesture) -> i8 {
        gesture.warmth()
    }
    fn memory(&self) -> &Memory {
        &self.memory
    }
    fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
    }
    fn remember(&mut self, _e: Episode) {}
    fn hear(&mut self, _h: Heard) {}
}

#[cfg(test)]
mod tests {

    #[test]
    fn every_gesture_is_in_all() {
        assert!(Gesture::ALL.iter().all(|g| g.listed()));
    }

    use super::*;

    fn episode(with: u16, peak: i8, end: i8) -> Episode {
        Episode {
            tick: Tick(0),
            with: VillagerId(with),
            frame: Frame::SmallTalk,
            peak,
            end,
            they_seemed: 0,
            notes: Vec::new(),
        }
    }

    #[test]
    fn regard_is_read_from_episodes_with_the_bad_weighing_double() {
        let mut m = Memory::default();
        assert_eq!(m.regard(VillagerId(1)), Regard::default());
        // One mildly bad talk reads as mildly bad, not as cold.
        m.remember(episode(1, -1, -1));
        assert_eq!(m.regard(VillagerId(1)).feeling, -1);
        // Two good ones for each bad one bring it back to neutral, not to warm.
        m.remember(episode(1, 1, 1));
        m.remember(episode(1, 1, 1));
        assert_eq!(m.regard(VillagerId(1)).feeling, 0);
        m.remember(episode(1, 2, 2));
        assert_eq!(m.regard(VillagerId(1)).feeling, 0);
        m.remember(episode(1, 2, 2));
        assert_eq!(m.regard(VillagerId(1)).feeling, 1);
        assert_eq!(m.regard(VillagerId(1)).familiarity, 5);
        // Someone else's episodes do not count.
        assert_eq!(m.regard(VillagerId(2)).familiarity, 0);
    }

    #[test]
    fn rounding_is_half_away_from_zero() {
        assert_eq!(rounded(10, 18), 1);
        assert_eq!(rounded(8, 18), 0);
        assert_eq!(rounded(-4, 4), -1);
        assert_eq!(rounded(-3, 4), -1);
        assert_eq!(rounded(-1, 4), 0);
    }

    #[test]
    fn frames_say_what_belongs_and_what_is_script() {
        assert!(!Frame::Transaction.allows(Gesture::Disclose));
        assert!(Frame::Confiding.allows(Gesture::Disclose));
        // Failures are failures inside the frame, never breaches of it.
        assert!(Frame::Greeting.allows(Gesture::Jab));
        assert!(Frame::Transaction.scripted(Gesture::Request));
        assert!(!Frame::SmallTalk.scripted(Gesture::Ask));
        assert!(Frame::SmallTalk.bids_to(Frame::Confiding));
        assert!(!Frame::Transaction.bids_to(Frame::Confiding));
    }

    #[test]
    fn telling_a_worry_is_remembered_per_person() {
        let mut m = Memory::default();
        let mut e = episode(1, 1, 1);
        e.tick = Tick(100);
        e.notes.push("told money".into());
        m.remember(e);
        let money = Topic::new("money");
        assert!(m.told_since(VillagerId(1), &money, Tick(50)));
        assert!(!m.told_since(VillagerId(1), &money, Tick(200)));
        assert!(!m.told_since(VillagerId(2), &money, Tick(50)));
    }
}
