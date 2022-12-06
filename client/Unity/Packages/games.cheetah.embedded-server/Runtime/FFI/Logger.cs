using System.Runtime.InteropServices;
using Games.Cheetah.EmbeddedServer.API;

namespace Games.Cheetah.EmbeddedServer.FFI
{
    public static class Logger
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
        public static extern void SetMaxLogLevel(EmeddedServerLogLevel cheetahEmeddedServerLogLevel);

        /**
         * Забрать и удалить из нативной части клиента существующие логи
         */
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "collect_logs")]
        public static extern void CollectLogs(LogCollector collector);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void LogCollector(EmeddedServerLogLevel cheetahEmeddedServerLogLevel, [MarshalAs(UnmanagedType.LPWStr)] string s);
    }
}