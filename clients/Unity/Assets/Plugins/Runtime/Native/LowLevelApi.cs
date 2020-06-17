using System.Runtime.InteropServices;

namespace Cheetach.Relay
{
    public static class LowLevelApi
    {
#if UNITY_IOS || UNITY_TVOS
        private const string Import = "__Internal";
#else
        private const string Import = "libcheetah_relay_client";
#endif


// pub extern "C" fn collect_logs(on_log_message: fn(*const c_char));
// pub unsafe extern "C" fn create_client(addr: *const c_char, room_hash: *const c_char, client_hash: *const c_char) -> u16;
// pub extern "C" fn get_connection_status<F, E>(client_id: u16, on_result: F, on_error: E);
// pub extern "C" fn receive_commands_from_server<F, E>(client_id: u16, collector: F, on_error: E);
// pub extern "C" fn send_command_to_server<E>(client_id: u16, command: &CommandFFI, on_error: E);
// pub extern "C" fn destroy_client(client_id: u16)

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void LogCollector(LogLevel logLevel, [MarshalAs(UnmanagedType.LPWStr)] string s);

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "init_logger")]
        public static extern void InitLogger();

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_max_log_level")]
        public static extern void SetMaxLogLevel(LogLevel logLevel);

        [DllImport(dllName: Import, CallingConvention = CallingConvention.Cdecl, EntryPoint = "collect_logs")]
        public static extern void CollectLogs(LogCollector collector);
    }
}