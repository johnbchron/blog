+++
title = "Migration to a new topics visibility model"
date = "2026-07-14"
hidden = true
+++

## Invitation Mechanism
- The "invitation" semantically means "I'm inviting you to pray for this topic".
- Only topic hosts can invite a user to a topic, and a given host can only invite their own friends.
- The actual technical meaning of an invitation changes based on the visibility setting of the topic.
	- For "public" and "friends", it's almost just a notification. For "private", it's also a user status.
	- On the "public" and "friends" visibility settings:
		- An invitation results in a notification for the invited user and a CTA to subscribe to the topic.
		- Example notification copy: "\[user\] has invited you to be a part of their topic \[topic title\]. Subscribe to receive updates".
	- On the "private" visibility setting:
		- An invitation results in a notification for the invited user and a CTA to view the topic, and additionally their invited-user status grants them access to see and interact with the topic. See the private topic invite list notes.
		- Example notification copy: "\[user\] has invited you to be a part of their private topic \[topic title\]. Tap to view".
- Private topic invite list:
	- Since invitations to a private topic also carry authorization to view and interact with the topic, that access must also be able to be rescinded.
	- Under the topic visibility settings, if the visibility is set to private, there will be another list containing the list of currently invited users.
	- Topic hosts may remove users from this list, which will rescind their access to the topic.
	- Edge cases:
		- If a host rescinds access to a user who is not their friend, they will not be free to re-invite that user. A host which is friends with the user must invite them again.
		- When a topic is switched from "public" or "friends" to "private", we must have UI to allow the user to pick which invited users will keep their invited status, or all or none. Otherwise by default, all invited users would have access to the topic.
- I'm wondering if we need specific UI to list private topics that the user has been invited to?

I think for now, to reduce complexity, we keep all prayers as invisible regardless of configuration or topic visibility.

## Invitation Implementation
- Under the hood, all invitations will be homogeneous.
- There will be an invitation table, keyed on user ID and topic ID.
- The data carried is essentially just a boolean "invited", which defaults to false if no row exists, plus some metadata.
- For a topic with any visibility setting, if the user has never been invited before, a row is created and "invited" is set to true.
- For "public" and "friends"-visible topics, once invited, the row will not be mutated.
- For "private" topics, if an invited user's access is rescinded in the UI, the "invited" field is set to false.

## Topic Content Visibility Matrix

| Scenario vs. Is Content Visible  | Topic page | Card on host's profile | "Created" event | "Update" event  | "Answered" event | "Self content" (prayers prayed) |
| -------------------------------- | ---------- | ---------------------- | --------------- | --------------- | ---------------- | ------------------------------- |
| "Public" topic                   | Yes        | Yes                    | No              | When subscribed | When subscribed  | Yes                             |
| "Friends" topic when not friends | No         | No                     | No              | No              | No               | Maybe?                          |
| "Friends" topic when friends     | Yes        | Yes                    | Yes             | When subscribed | When subscribed  | Yes                             |
| "Private" topic when not invited | No         | No                     | No              | No              | No               | Maybe?                          |
| "Private" topic when invited     | Yes        | Maybe?                 | Yes             | When subscribed | When subscribed  | Yes                             |

## Subscription Mechanism
- The primary purpose of subscriptions is to gate notifying a user about topic updates on their affirmative action.
	- I.e. if they haven't consented to anything or taken action on a topic, they should receive a limited amount of feed content on that topic and should not be notified about it.
- Subscribing to a topic means semantically that the user is requesting to be notified about it.
- Notifying the user in upholding a subscription takes two forms:
	- "Update" and "answered" events appear in the user's topic feed.
	- They optionally receive push notifications for "update" and "answered" topic events.
- A user subscribing to a topic is the only way that it enters their field of view again after it was created.
- Subscriptions are simple in regards to visibility, in that they only propagate existing topic visibility.
	- A subscription never modifies a topic's apparent visibility.
	- If the topic is already visible to the user, subscribing to it simply increases the content reach of that topic for that user.
	- If the topic is not visible to the user, a subscription means nothing.
- UI is required to list all a user's subscribed topics, so as to audit or bulk-unsubscribe.
- Open questions:
	- Are there any circumstances, aside from the migration, where we'd need to auto-add subscriptions to users?

## Subscription Implementation
- Subscriptions only really need a boolean switch per user and topic.
- We can simply store subscriptions in an array on the user row, where a present entry indicates a subscription
	- The data model is so simple (just a boolean).
	- It's a user-local interaction (subscribing doesn't interact with any other users).
	- Our queries will benefit from the data being co-located.
- Notifications can be executed without the expo notification center, instead by deeplinking to the topic's page or the "update" or "answered" event on the feed page.

## Topic Visibility Settings Migration
- If a topic is set to "public on flock", transition to the new "public".
- If a topic is set to "shared with select friends", transition to the new "private", and mark those users as invited.
- If a topic is set to "just me", transition to the new "private".
