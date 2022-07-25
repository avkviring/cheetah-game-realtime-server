#if UNITY_ANDROID
namespace Cheetah.User.Accounts.Google
{
    /// <summary>
    /// Результат авторизации
    /// </summary>
    public readonly struct GoogleAuthenticationResult
    {
        public GoogleAuthenticationResult(bool registeredPlayer, User user, string userDisplayName)
        {
            User = user;
            RegisteredPlayer = registeredPlayer;
            UserDisplayName = userDisplayName;
        }

        /// <summary>
        /// true - новый игрок, false - существующй игрок
        /// </summary>
        public bool RegisteredPlayer { get; }

        /// <summary>
        /// Авторизованный игрок
        /// </summary>
        public User User { get; }

        /// <summary>
        /// Имя игрока из Google Play Games
        /// </summary>
        public string UserDisplayName { get; }
    }
}
#endif