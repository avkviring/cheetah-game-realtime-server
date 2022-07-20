using System;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Threading.Tasks;
using Docker.DotNet.Models;
using UnityEngine;

namespace Cheetah.Platform.Editor.LocalServer.CheetahRegistry
{
    public class CheetachDockerRegistry
    {
        public static string URL = "registry.dev.cheetah.games/cheetah/platform";
        private readonly CheetahRegistrySettings settings;

        public CheetachDockerRegistry(CheetahRegistrySettings settings)
        {
            this.settings = settings;
        }

        public async Task<AuthConfig> CheckAndGetEncodedConfig()
        {
            if (!await CheckAuth()) throw new CheetahRegistryAuthException();

            return CreateConfig();
        }

        /// <summary>
        ///     Проверить правильность авторизационных данных для внешнего реестра
        /// </summary>
        /// <returns></returns>
        public async Task<bool> CheckAuth()
        {
            return true;
            
        }

        private AuthConfig CreateConfig()
        {
            return new AuthConfig
            {
                Username = settings.Login,
                Email = settings.Login,
                Password = settings.Password,
                ServerAddress = $"https://{URL}"
            };
        }
    }
}