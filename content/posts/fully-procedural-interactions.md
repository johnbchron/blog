---
title: "Fully Procedural Interactions in Games"
written_on: "2024.08.08"
public: true
---

I think a fully procedural game would be super cool. By that I mean a game that simulates as many interactions as possible, particularly between the player and NPCs, and between individual NPCs. Also a system where as many actions as possible are taken by NPCs as a result of human-like decision making, and then dialog is synthesized on top of all that data.

For example, consider the following cases:
1. A street vendor notices that you're wearing armor, deduces that you're a warrior, and offers you a set of prices and some small-talk dialog.
2. You take off your helm just before visiting the street vendor, they see your dog-like ears and offer you a different set of prices. Their dialog is also different.
3. You take off your helm after being offered the "warrior" prices and dialog, and the street vendor has a visible reaction but doesn't change their behavior because their pride matters to them.

This is a little microcosm, but it illustrates how a player's small actions influence how NPCs interact with them. The above interaction would have gone differently if:
- The NPC's vision and perceptiveness were better, and could tell from the start that the player was a dog-person.
- The NPC's capacity for deception and suavity were better, and could find an excuse to raise or lower prices based on their new realization without offending their pride.
- The NPC's self respect were low enough, and they didn't care that their racial bias would be noticed if they changed their prices after realizing the player's racial identity, positively or negatively.
- The NPC's racial bias was strong enough to overcome their pride and change their prices, positively or negatively.
- The NPC had met the player before, and could recognize the player's identity after they took off their helm.
- The NPC had a sensitive enough nose, and could recognize that the beast stench coming off the player was from the wyverns that had been raiding farmers nearby.

All of these interactions are distinct and have different flavors, despite some of them having similar outcomes.

I suppose that the desired behavior here is DnD-level interaction variability, but using a system that **1)** works in a game and **2)** can be scaled to NPC-to-NPC interaction.

The factors involved here are pretty simple, and most of them can be described with single-value coefficients: `visual_acuity`, `social_perception`, `self_respect`, `racial_bias['dog_people']`, `has_met_player`, `olfactory_accuracy`. The difficult part here is the dialog that results from the interactions.

Opting for a no-dialog solution is an option, but it's boring and, in my opinion, throws away the coolness gained by this extremely reactive architecture. I don't really have a solution thought up for this; I'm just playing around with ideas for now.

One potential solution is to enumerate all possible dialog interaction types, and build a templating system to get a baseline for writing dialog text. This is functional, but really boring.

Given that all the interactions are data-driven and we have a dialog baseline, I think it could be a viable to have an LLM come in, take the existing dialog, and spice it up using  background data and the interaction data as context, with some relevant "knowledge-base" entries injected as well.

This wouldn't be possible without the dialog baseline, because we all know that LLMs are terrible at writing dialog from exotic perspectives "from scratch". By changing the task from "generate" to "spicify", I think an LLM would be viable.

I'll experiment with this and write further on my findings.
