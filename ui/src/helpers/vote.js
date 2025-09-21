import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export async function createVote(user, guildId, title, description, channelId) {
  await axios.post(`${VITE_KB_API_URL}/guilds/${guildId}/votes`, {
    'title': title,
    'description': description,
    'channel_id': channelId,
    'options': [
      {
        "label": "Happy",
        "emoji": {
          "name": "😊"
        }
      },
      {
        "label": "Less Happy"
      },
      {
        "label": "Decline to Answer",
        "emoji": {
          "animated": false,
          "id": "592082557961633877",
          "name": ":RedCross:"
        }
      }
      ],
    'role_list_type': "BLACKLIST",
    'role_list': []
    },
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}

export async function closeVote(user, guildId, messageId) {
  await axios.post(`${VITE_KB_API_URL}/guilds/${guildId}/votes/${messageId}/close`, {},
    {
      headers: {
        'Authorization': 'Discord ' + user.token.accessToken
      }
    }
  )
}