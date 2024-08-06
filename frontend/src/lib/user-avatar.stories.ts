import type { Meta, StoryObj } from "@storybook/svelte";
import UserAvatar from "./user-avatar.svelte";

const meta = {
  title: "Koso/UserAvatar",
  component: UserAvatar,
  tags: ["autodocs"],
} satisfies Meta<UserAvatar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Main: Story = {
  args: {
    user: {
      name: "Shad",
      email: "shad@gmail.com",
      picture:
        "https://lh3.googleusercontent.com/a/ACg8ocIRfl1MJrdKF_V8e46SQijmFzs1JoEaQLogCsOEIYC-T2Hk2xcPKw=s96-c",
      exp: 0,
    },
  },
};
