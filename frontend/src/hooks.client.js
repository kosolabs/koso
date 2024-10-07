/** @type {import('@sveltejs/kit').HandleClientError} */
export async function handleError({ error, event, status, message }) {
  const errorId = Date.now().toString(16);
  console.error(
    `An error occurred: (${status}) ${message} -- ${error} (id=${errorId})`,
    event,
  );

  return {
    message: `${error} (id=${errorId})`,
  };
}
