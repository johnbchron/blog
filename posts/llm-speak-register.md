+++
title = "LLM-Speak as a Coping Mechanism"
date = "2026-04-14"
+++

I've had this reoccurring thought that the characteristic literary register of modern LLMs is a sort of coping mechanism for the LLM. Speaking in short fragments is simpler and easier than constructing fully-breaded sentences. I have a few suspicions as to what benefits this yields for an LLM.

Firstly, I believe that especially in longer, more deliberative writing, producing simpler sentences may allow an LLM to recover its own context better. Consider token generation that involves multiple paragraphs, in which each paragraph has a distinct goal. I think LLM-speak allows the LLM to have a better sense of what it's already written, i.e. simpler writing is easier to build on. The same applies to prolific use of header-level text, bolded titles at the beginnings of bullet points, emojis, and other consistent overuse of Markdown features. I think these help the LLM remember both what it's actually talking about and emphasizing, at the token level.

Secondly, it may be that LLM-speak ends up being some sort of gradient-descent local minimum of optimizing for token count, intelligibility, broad appeal, conveyance, and simplicity. This is much more of an intuition as I lack knowledge of the inner workings of LLM training, but it may be that within the LLM's reinforcement training, this kind of style ends up being close enough to everything while also letting the LLM be "lazy" in a sense. It seems plausible to me that architecturally, an LLM will resolve to roughly a single literary register for all the prose for which it isn't given explicit instructions.

I'd love to see more research on the register and tone of LLMs. Experiments could include testing whether an LLM can maintain argumentative cohesion when continuing inputs written in a more human - and therefore necessarily more obfuscated - register.
