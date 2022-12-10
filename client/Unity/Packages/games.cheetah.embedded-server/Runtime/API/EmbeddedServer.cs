using System;
using System.Net;
using System.Net.Http;
using Games.Cheetah.EmbeddedServer.FFI;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using static Games.Cheetah.GRPC.Internal.Internal;
using Logger = Games.Cheetah.EmbeddedServer.FFI.Logger;

#if UNITY_5_3_OR_NEWER
using UnityEngine;
using AOT;

#else
using Serilog;
#endif

#nullable enable
namespace Games.Cheetah.EmbeddedServer.API
{
    /// <summary>
    /// Класс для запуска встроенного realtime сервера, в основном используется для тестов.
    /// Однако может использоваться и для production целей.
    ///
    /// Порядок использования:
    ///  - создаем экземпляр сервера (можно один сервер, на один тест)
    ///  - создаем комнату
    ///  - создаем необходимое количество пользователей в команте
    ///  - соединяемся с сервером от имени клиента(ов) для тестирования
    ///  - удаляем сервер для освобождения ресурсов
    /// </summary>
    public class EmbeddedServer
    {
        private readonly Server.Description description;
        private static string? errorMessage;

        public EmbeddedServer(IPAddress bindAddress)
        {
            unsafe
            {
                var bindFFIAddress = new Server.BindAddress();
                var addressBytes = bindAddress.GetAddressBytes();

                bindFFIAddress.bindAddress[0] = addressBytes[0];
                bindFFIAddress.bindAddress[1] = addressBytes[1];
                bindFFIAddress.bindAddress[2] = addressBytes[2];
                bindFFIAddress.bindAddress[3] = addressBytes[3];

                if (!Server.RunNewServer(ref description, OnError, ref bindFFIAddress))
                {
                    throw new Exception("Cannot run embedded server. " + errorMessage);
                }
            }
        }


        public InternalClient CreateGrpcClient()
        {
            var channel = GrpcChannel.ForAddress(
                GetInternalWebGrpcUri(), new GrpcChannelOptions
                {
                    HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
                }
            );
            return new InternalClient(channel);
        }

        public void Destroy()
        {
            if (!Server.DestroyServer(description.id))
            {
                throw new Exception("Embedded server not found");
            }
        }

        public string GetUdpGameHost()
        {
            unsafe
            {
                return $"{description.gameIp[0]}.{description.gameIp[1]}.{description.gameIp[2]}.{description.gameIp[3]}";
            }
        }

        public ushort GetUdpGamePort()
        {
            return description.gamePort;
        }

        public Uri GetAdminWebGrpcUri()
        {
            unsafe
            {
                var ip =
                    $"{description.admin_webgrpc_ip[0]}.{description.admin_webgrpc_ip[1]}.{description.admin_webgrpc_ip[2]}.{description.admin_webgrpc_ip[3]}";
                return new Uri($"http://{ip}:{description.admin_webgrpc_port}");
            }
        }

        public Uri GetInternalGrpcUri()
        {
            unsafe
            {
                var ip =
                    $"{description.internal_grpc_ip[0]}.{description.internal_grpc_ip[1]}.{description.internal_grpc_ip[2]}.{description.internal_grpc_ip[3]}";
                return new Uri($"http://{ip}:{description.internal_grpc_port}");
            }
        }

        public Uri GetInternalWebGrpcUri()
        {
            unsafe
            {
                var ip =
                    $"{description.internal_webgrpc_ip[0]}.{description.internal_webgrpc_ip[1]}.{description.internal_webgrpc_ip[2]}.{description.internal_webgrpc_ip[3]}";
                return new Uri($"http://{ip}:{description.internal_webgrpc_port}");
            }
        }


#if UNITY_5_3_OR_NEWER
        [MonoPInvokeCallback(typeof(Server.OnServerError))]
#endif
        private static void OnError(string? message)
        {
            errorMessage = message;
        }


        public static void InitLogger(EmeddedServerLogLevel emeddedServerLogLevel)
        {
            Logger.InitLogger();
            Logger.SetMaxLogLevel(emeddedServerLogLevel);
        }

        public static void ShowCurrentLogs()
        {
            Logger.CollectLogs(ShowLog);
        }

#if UNITY_5_3_OR_NEWER
        [MonoPInvokeCallback(typeof(Logger.LogCollector))]
        private static void ShowLog(EmeddedServerLogLevel level, string log)
        {
            switch (level)
            {
                case EmeddedServerLogLevel.Info:
                    Debug.Log(log);
                    break;
                case EmeddedServerLogLevel.Warn:
                    Debug.LogWarning(log);
                    break;
                case EmeddedServerLogLevel.Error:
                    Debug.LogWarning(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }
#else
        private static void ShowLog(EmeddedServerLogLevel level, string log)
        {
            switch (level)
            {
                case EmeddedServerLogLevel.Info:
                    Log.Information(log);
                    break;
                case EmeddedServerLogLevel.Warn:
                    Log.Warning(log);
                    break;
                case EmeddedServerLogLevel.Error:
                    Log.Error(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }
#endif
    }
}