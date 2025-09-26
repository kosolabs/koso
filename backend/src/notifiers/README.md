# Microsoft Teams Integration

Koso supports sending notifications to Microsoft Teams channels using Microsoft 365 Agents (the modern replacement for deprecated Incoming Webhooks).

## Setup

1. **Create a Microsoft 365 Agent**:

   - Use the Microsoft 365 Agents Toolkit to create a notification bot
   - Configure the bot to post messages to Teams channels
   - Obtain the bot token and channel ID for your target channel

2. **User Authorization**:
   - Users can authorize Teams notifications through their profile page
   - Click "Connect Microsoft Teams" button in the Teams section
   - Enter your bot token and channel ID in the form
   - Click "Connect Teams" to complete the authorization
   - The credentials are stored securely in the user's notification settings

## How It Works

- Teams notifications use Microsoft Graph API to send messages to specific channels
- Each user can configure their own bot token and channel ID for their preferred channel
- Messages are sent using the modern Teams messaging API
- Supports Adaptive Cards and rich message formatting

## Getting Your Bot Token and Channel ID

1. **Create a Microsoft 365 Agent**:

   - Use the Microsoft 365 Agents Toolkit (previously Teams Toolkit)
   - Follow the [official documentation](https://learn.microsoft.com/en-us/microsoftteams/platform/toolkit/overview)
   - Deploy your agent to Teams

2. **Obtain Bot Token**:

   - After deployment, your agent will have a bot token
   - This is typically found in your agent's configuration or Azure Bot Service
   - The token allows Koso to authenticate with Microsoft Graph API

3. **Find Channel ID**:
   - In Teams, right-click on your target channel
   - Select "Get link to channel"
   - The channel ID is in the URL: `https://teams.microsoft.com/l/channel/19:abc123...`
   - Extract the part after `/l/channel/` as your channel ID

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
