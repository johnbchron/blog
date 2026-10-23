+++
title = "A Proposition for Propia"
date = "2026-10-23"
+++

I'm considering building a print server, something to replace the likes of
PaperCut.

For those who are unfamiliar, PaperCut is a print queue software, which does
some combination of 1) tracking printer usage, 2) hold-and-release for print
jobs (find-me printing), 3) cost tracking, 4) print job transformation, and
other things. The exact features depend on how you configure it.

I'm dissatisfied with the *messiness* of PaperCut, and the lack of care applied
to this class of product both from PaperCut and other similar vendors. I know
this is partially consumer bias, but it feels as though companies have not
applied much rigor to the elimination of edge cases and undefined behavior in
these products.

Also, given this particular moment in history, an opportunity is emerging to
build an IPP-only server as a solution within this problem space. Every major
client family now supports IPP as a first-class protocol, and every major
printer family does the same. Several large families of tech are transitioning
away from other methods, such as Windows pushing to Protected Print Mode or CUPS
deprecating printer drivers in favor of IPP. The tides have shifted in favor
of IPP-native.

So my self-proposition is to build such a product based on an IPP server. I know
that it won't be feasible to use purely IPP all the way end-to-end, but I would
aim to build the non-IPP items on top of IPP instead of IPP on top of a legacy
stack. Rust would be my language of choice for a number of reasons which I am
too lazy to enumerate here.

The future looks bright for such a product, at least to my star-struck eyes.

I think I'll name it **Propia**, after the Latin adjective.

Oh, and no AI-generated code.
