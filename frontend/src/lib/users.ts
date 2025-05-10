export type User = {
  email: string;
  name: string;
  picture: string;
};

export type FullUser = User & {
  premium: boolean;
};
