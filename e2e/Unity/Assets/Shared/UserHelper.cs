using System.Threading.Tasks;
using Cheetah.Matches.Matchmaking;
using Cheetah.Matches.Matchmaking.GRPC;
using Cheetah.Platform;
using Cheetah.User.Accounts.Cookie;

namespace Shared
{
    public class UserHelper
    {
        /// <summary>
        /// Создаем нового игрока и закидываем его в битву
        /// </summary>
        /// <param name="clusterConnector"></param>
        /// <returns></returns>
        public static async Task<TicketResponse> CreateNewPlayerAndMatchToBattle(ClusterConnector clusterConnector, string prefix)
        {
            // создаем нового пользователя
            var cookieAuthenticator = new CookieAuthenticator(clusterConnector, prefix);
            cookieAuthenticator.RemoveLocalCookie();
            var loginOrRegister = await cookieAuthenticator.LoginOrRegister();

            // сообщаем mm о желании попасть в битву
            var player = loginOrRegister.User;
            var scheduler = new MatchmakingScheduler(player);
            var ticket = await scheduler.Schedule("gubaha", UserGroup);
            return ticket;
        }

        public const ulong UserGroup = 8;
    }
}