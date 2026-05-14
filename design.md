# Think Alike
Think alike it's a platform where you can publish your thoughts and find people who think alike.

## Tech Stack
- Frontend: pnpm + Vue 3 + TypeScript + Element Plus
- Backend: Rust + axum
- Database: PostgreSQL + pgvector extension
- Embedding: OpenAI's Embedding API

We should produce a Dockerfile in the end. Be careful with security.

## Pages
### Home Page
To enter this page, we require the user to log in with their GitHub account. And we'll have a whitelist or blacklist mechanism which blocks untrusted users.

#### Publish
People can probably use their thoughts here, and each thought should have a title and a description. After the user publishs a thought, we'll use the OpenAI's Embedding API to embed the thought into a vector space. Then we'll store the vector in the database.

Note that we do need rate limit. A user can only publish up to 30 thoughts a day.

#### Find
Here the users can choose one thought and then we show a graph which the center is the chosen thought. We search the thought in our database and show relevant thoughts in the graph. The more similar the thought is, the closer it is in the graph.

## Kanban Page
Here we cluster all the thoughts according to their embedding vector. And then we show a dynamic graph, which all the thoughts floating in the space. Newer thoughts have a higher probability to be shown. Again, the distance between the nodes in the graph is the distance between the thoughts. Do show the title in each note.