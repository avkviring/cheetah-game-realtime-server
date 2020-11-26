using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class LoggerExternals
    {
        /**
         * Инициализировать логер нативной части (без вызова этой функции логи собираться не будут)
         */
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "init_logger")]
        public static extern void InitLogger();

        /**
         * Установить уровень логирования в нативной части клиента
         */
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_max_log_level")]
        public static extern void SetMaxLogLevel(LogLevel logLevel);

        /**
         * Забрать и удалить из нативной части клиента существующие логи
         */
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "collect_logs")]
        public static extern void CollectLogs(LogCollector collector);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void LogCollector(LogLevel logLevel, [MarshalAs(UnmanagedType.LPWStr)] string s);
    }
}