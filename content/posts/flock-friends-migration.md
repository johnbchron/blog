+++
title = "Migration to a symmetric, explicit friends model."
date = "2026-06-27"
hidden = true
+++

## Notes on Friends:
- We'd encode friends with a separate and explicit table, i.e. a `friends` table with `user_a` and `user_b` columns.
### Friending Process
- `alice` and `bob` are not yet friends. The CTA they see says "Friend".
- User `alice` sends a friend request to user `bob`. CTA switches to "Requested" for `alice` and "Accept" for `bob`.
- `bob` gets a notification that `alice` sent a friend request.
- Multiple outcomes:
	- `bob` accepts the friend request, and they are now friends. The CTA disappears.
	- `bob` declines the friend request. The CTA switches back to "Friend" for both of them. `alice` cannot request again for a week, but `bob` may send `alice` a friend request anytime.

## Visibility Rules
### Topics
- I want to introduce the concept of "subscribing" to a topic. This will reduce noise for users and move us closer to a consent-centric approach.
	- To subscribe to a topic means you get its "updated/answered" events.
	- If you're not subscribed to a topic, you don't see these events.
	- Subscribing is only allowed if you can view the topic to begin with.
- Topic visibility modes:

| Mode    | Who can view           | Who sees the "created" event |
| ------- | ---------------------- | ---------------------------- |
| Public  | Everyone on Flock      | Friends of user              |
| Friends | Friends of user        | Friends of user              |
| Private | Nobody, unless invited | Nobody                       |
- I want to introduce the concept of "inviting" someone to a topic. This is similar to the existing feature of "sharing" a topic with a user, but is closer to the consent-centric approach we want.
	- To invite a user to a topic means you're inviting them to view a topic.
	- You can only invite friends to a topic, and the topic must be private.
	- Before they accept, they can only see the title of the topic, the description, and the user inviting them.
	- After they accept, they can see all the contents of the topic, and they are automatically subscribed to it.
### Prayers
- We can divide prayer visibility into two categories:
	- Personally shared prayers:
		- These are always only visible to the user praying and the user being prayed for.
	- Topic prayers:
		- I want to consider allowing the topic hosts to set prayer visibility.
			- If set to "visible", other users who can view the topic can view prayers.
			- If set to "invisible", only the user praying and the topic hosts can view a given prayer.
### Dailies
- A daily is visible to a user's friends.
### Users
- A user's profile is generally visible to another user.
- The topics visible on a user's profile are governed by the topic visibility modes above.
- A user's daily is visible to their friends.
