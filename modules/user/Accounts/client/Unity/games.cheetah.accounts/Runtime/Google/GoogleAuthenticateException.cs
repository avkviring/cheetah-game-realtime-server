#if UNITY_ANDROID
using System;
using GooglePlayGames.BasicApi;

namespace Cheetah.Accounts.Google
{
    public class GoogleAuthenticateException : Exception
    {
        public GoogleAuthenticateException(SignInStatus status) : base(status.ToString())
        {
            Status = status;
        }

        public SignInStatus Status { get; }
    }
}
#endif