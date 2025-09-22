import axios from "axios";

const VITE_KB_API_URL = import.meta.env.VITE_KB_API_URL

export async function createVote(user, guildId, title, description, channelId, options, isMultiSelect,
                                 roleListType, roleList) {
  await axios.post(`${VITE_KB_API_URL}/guilds/${guildId}/votes`, {
    'title': title,
    'description': description,
    'channel_id': channelId,
    'options': options,
    'is_multi_select': isMultiSelect,
    'role_list_type': roleListType,
    'role_list': roleList,
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