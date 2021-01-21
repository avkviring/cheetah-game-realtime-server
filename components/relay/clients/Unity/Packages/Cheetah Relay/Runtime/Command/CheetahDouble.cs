using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class CheetahDouble
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, double value);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_float_value_listener")]
        public static extern bool SetListener(Listener listener);


        /// <summary>
        /// Установить значение
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="value"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_float_value")]
        public static extern bool Set(ref CheetahObjectId objectId, ushort fieldId, double value);

        /// <summary>
        /// Инкрементация значения
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="value"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_float_value")]
        public static extern bool Increment(ref CheetahObjectId objectId, ushort fieldId, double increment);
    }
}