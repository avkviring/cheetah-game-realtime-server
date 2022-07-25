namespace Cheetah.User.Accounts.Cookie
{
    public class CookieAuthenticationResult
    {
        public CookieAuthenticationResult(bool registered, User user)
        {
            Registered = registered;
            User = user;
        }

        /// <summary>
        /// true - новый игрок, false - существующй игрок
        /// </summary>
        public bool Registered { get; }

        /// <summary>
        /// Авторизованный игрок
        /// </summary>
        public User User { get; }
    }
}