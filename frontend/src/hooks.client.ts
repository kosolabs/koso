import type { HandleClientError } from "@sveltejs/kit";

export const handleError: HandleClientError = async ({
  error,
  event,
  status,
  message,
}) => {
  const errorId = Date.now().toString(16);
  console.error(
    `An error occurred: (${status}) ${message} -- ${error} (id=${errorId})`,
    event,
  );

  return {
    message: `${error} (id=${errorId})`,
  };
};
