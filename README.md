# create-svelte

Everything you need to build a Svelte project, powered by [`create-svelte`](https://github.com/sveltejs/kit/tree/main/packages/create-svelte).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```bash
# create a new project in the current directory
npm create svelte@latest

# create a new project in my-app
npm create svelte@latest my-app
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://kit.svelte.dev/docs/adapters) for your target environment.

# Backend

## Developing

Start a development server:

```bash
cargo run
```

You can interact with the server at http://localhost:3000 using a browser, a CLI tool like cURL or an API explorer such as [Postman](https://www.postman.com/downloads/).

List tasks:

```bash
curl localhost:3000/task/list | jq
```

Create a task:

```bash
curl -H 'Content-Type: application/json' \
      -d "{\"name\": \"$USER task $(date '+%d/%m/%Y_%H:%M:%S')\"}" \
      -X POST \
      localhost:3000/task/create | jq
```