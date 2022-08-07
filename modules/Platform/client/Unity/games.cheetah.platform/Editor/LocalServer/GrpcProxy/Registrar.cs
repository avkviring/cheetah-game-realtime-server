using System.Collections.Generic;
using UnityEditor;

namespace Cheetah.Platform.Editor.LocalServer.GrpcProxy
{
    /// <summary>
    ///  Фабрика для создания:
    /// - контейнера с nginx для проксирования GRPC/HTTP запросов к сервисам
    /// - UI для управления конфигурацией nginx и общими параметрами сервисов
    /// </summary>
    [InitializeOnLoad]
    public static class Registrar
    {
        static Registrar()
        {
            var ui = new GrpcProxyApplicationConfigurator();
            Registry.Register("grpc_proxy",
                new List<ServerApplication>() {new GrpcProxyApplication(ui)},
                new List<IApplicationsConfigurator> {ui}
            );
        }
    }
}