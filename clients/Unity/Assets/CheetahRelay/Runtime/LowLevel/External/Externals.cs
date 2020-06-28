using System.Runtime.InteropServices;

namespace CheetahRelay.Runtime.LowLevel.External
{
    public static class Externals
    {
#if UNITY_IOS || UNITY_TVOS
        private const string Import = "__Internal";
#else
        private const string Import = "libcheetah_relay_client";
#endif

        /**
         * Инициализировать логер нативной части (без вызова этой функции логи собираться не будут)
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "init_logger")]
        public static extern void InitLogger();

        /**
         * Установить уровень логирования в нативной части клиента
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_max_log_level")]
        public static extern void SetMaxLogLevel(LogLevel logLevel);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void LogCollector(LogLevel logLevel, [MarshalAs(UnmanagedType.LPWStr)] string s);

        /**
         * Забрать и удалить из нативной части клиента существующие логи
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "collect_logs")]
        public static extern void CollectLogs(LogCollector collector);


        /**
         * Создать клиента, после создания клиент начнет процесс соединения с сервером
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern ushort CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            [MarshalAs(UnmanagedType.LPStr)] string roomHash,
            [MarshalAs(UnmanagedType.LPStr)] string clientHash);


        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void NetworkStatusDelegate(NetworkStatus status);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void ErrorDelegate();

        /**
         * Получить статус сетевого соединения с серером
         *
         * errorDelegate вызывается в случае системной ошибки, например если такого клиента уже нет
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern void GetConnectionStatus(ushort client, NetworkStatusDelegate statusDelegate, ErrorDelegate errorDelegate);

        /**
         * Отправить команду на сервер
         * command - будет скопирован и его сразу же можно использовать для отправки следующей команды
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "send_command_to_server")]
        public static extern void SendCommandToServer(ushort clientId, in Command command);

        /**
         * Удалить клиента и закрыть соединение с сервером
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_client")]
        public static extern void DestroyClient(ushort client);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void ServerCommandCollector(in Command command);

        /**
         * Получить серверные команды
         */
        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "receive_commands_from_server")]
        public static extern void ReceiveCommandsFromServer(ushort client, ServerCommandCollector collector, ErrorDelegate errorDelegate);
    }
}