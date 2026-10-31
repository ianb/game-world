# The Welling: the scrap zone's people

Placeholders, all of them: names, roles, moods and programs are the agent's, for Ian to
rename, rewrite or replace. Content in `worlds/welling/content/villagers/`, programs in
`worlds/welling/characters/src/`.

| id | role | lives | moods | program |
| --- | --- | --- | --- | --- |
| ora | refiner at the Mire | the Mire | proud of refining, irritable in crowds | refines at a bench: two raw gas from the casks make one refined, every 90 minutes |
| sab | works the hood over the vent, refines after | the Mire | restless, uneasy in the dark | captures raw gas at the hood into the casks (up to 12), every 45 minutes; refines in the afternoon |
| tully | keeps the gas stall on the yard | a shanty | proud of earning, anxious under 20 coins | at 8:30 fetches refined gas from the Mire's hold to the stall, paying Ora 3 a measure; sells from 9 |
| grue | fisher who eats the catch raw | a shanty | lonely after 8 hours, cheered by rain | fishes off the shanty raft; eats raw fish at home |
| pike | hangs about the stall | a shanty | anxious under 10 coins, gloomy in rain, restless | loiters at the stall and the Kettle; will buy gas once substances exist |
| nessa | fisher who cooks | the cutter | cheered by fine days, gloomy in rain | fishes off the south raft; cooks two at a time on her stove; eats cooked fish |
| hobb | lasher | the tug | proud of the lashings, irritable when stuck | walks the bollards (yard, shanty raft, vent raft) |
| lark | runs the drinks at the Kettle | the Kettle | cheered by company, lonely after 4 hours | serves; drinkers pay her 2 a drink |

Shared behaviour (`toolbox::human`): the privy on the yard, a cistern for water (there are
no wells at sea), food at home when starving, a drink at the Kettle, and the mood
reactions (the lonely go to the Kettle when nobody is about).

Substances (`worlds/welling/content/substances/`, all numbers placeholders):

| substance | lift | lasts | moods | wakes | habit | can go wrong |
| --- | --- | --- | --- | --- | --- | --- |
| kelp | +1 | 3 h | gloomy -2, restless -1 | 8 | 3 | no |
| refined gas | +3 | 4 h | anxious -6, gloomy -4, cheerful +6 | 15 | 6 | no |
| raw gas | +1 | 1 h | | | 2 | a quarter of portions odd |
| raw fish | | | | | | 15% of portions odd, 10% of batches bad |
| drink | +1 | 2 h | lonely -2, anxious -1 | | | no |

Cooked fish is not a substance: cooking takes out what the seep put in. Everyone starts a
little hooked on kelp (Grue most), Pike hooked on refined gas, Sab on raw gas. Kelp is
kept at home and bought at the Kettle three at a time; Pike buys gas at the stall one
measure at a time and scrounges 5 coins a day.

A boat calls at the berth on Tuesdays and Fridays at ten and leaves at noon (the
timetabled line, `vehicle: boat`). It tops the berth's crates up to 40 kelp and 30
drink, and buys refined gas at 6 a measure. Ora sends the Mire's refined gas; Lark
fetches kelp and drink for the Kettle at 1 each and sells kelp at 2.
