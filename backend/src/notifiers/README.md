# Microsoft Teams Integration

Koso supports sending notifications to Microsoft Teams channels using Microsoft 365 Agents (the modern replacement for deprecated Incoming Webhooks).

## Setup

1. **Create a Microsoft 365 Agent**:

   - Use the Microsoft 365 Agents Toolkit to create a notification bot
   - Configure the bot to post messages to Teams channels
   - Obtain the bot token and channel ID for your target channel

2. **User Authorization**:
   - Users can authorize Teams notifications through their profile page
   - Each user provides their own bot token and channel ID for their preferred channel
   - The credentials are stored securely in the user's notification settings

## How It Works

- Teams notifications use Microsoft Graph API to send messages to specific channels
- Each user can configure their own bot token and channel ID for their preferred channel
- Messages are sent using the modern Teams messaging API
- Supports Adaptive Cards and rich message formatting

## Security

- Each user's bot token and channel ID are stored separately in their notification config
- No shared credentials between users
- Uses Microsoft's modern authentication and security standards
- No global secrets required - each user manages their own Teams integration

## Why Not Incoming Webhooks?

Microsoft has deprecated Incoming Webhooks in favor of Microsoft 365 Agents because:

- **Better Security**: Modern authentication and authorization
- **More Features**: Support for Adaptive Cards, rich formatting, and interactive elements
- **Future-Proof**: Part of Microsoft's ongoing Teams platform evolution
- **Better Integration**: Native Teams bot capabilities

## Troubleshooting

- Ensure the bot token is valid and has proper permissions
- Check that the channel ID is correct and accessible
- Verify Microsoft Graph API permissions are configured correctly
- Check server logs for any API errors when sending messages
