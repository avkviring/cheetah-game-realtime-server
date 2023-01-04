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

        public EmbeddedServer(
            IPAddress internalGrpcAddress, ushort internalGrpcPort,
            IPAddress internalWebGrpcAddress, ushort internalWebGrpcPort,
            IPAddress adminWebGrpcAddress, ushort adminWebGrpcPort,
            IPAddress gameUdpAddress, ushort gameUdpPort
        )
        {
            unsafe
            {
                var internalGrpcSocket = NewBindSocket(internalGrpcAddress, internalGrpcPort);
                var internalWebGrpcSocket = NewBindSocket(internalWebGrpcAddress, internalWebGrpcPort);
                var adminWebGrpcSocket = NewBindSocket(adminWebGrpcAddress, adminWebGrpcPort);
                var gameUdpSocket = NewBindSocket(gameUdpAddress, gameUdpPort);

                if (!Server.RunNewServer(ref description, OnError,
                        ref internalGrpcSocket,
                        ref internalWebGrpcSocket,
                        ref adminWebGrpcSocket,
                        ref gameUdpSocket))
                {
                    throw new Exception("Cannot run embedded server. " + errorMessage);
                }
            }
        }

        public EmbeddedServer(
            IPAddress address
        ) : this(address, 0, address, 0, address, 0, address, 0)
        {
        }


        private static unsafe Server.BindSocket NewBindSocket(IPAddress bindAddress, ushort port)
        {
            var result = new Server.BindSocket();
            var addressBytes = bindAddress.GetAddressBytes();
            result.port = port;
            result.bindAddress[0] = addressBytes[0];
            result.bindAddress[1] = addressBytes[1];
            result.bindAddress[2] = addressBytes[2];
            result.bindAddress[3] = addressBytes[3];
            return result;
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
                    Debug.Log("server:\t" + log);
                    break;
                case EmeddedServerLogLevel.Warn:
                    Debug.LogWarning("server:\t" + log);
                    break;
                case EmeddedServerLogLevel.Error:
                    Debug.LogWarning("server:\t" + log);
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