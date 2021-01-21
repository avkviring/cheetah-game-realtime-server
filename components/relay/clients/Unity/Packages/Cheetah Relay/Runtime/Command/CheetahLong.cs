using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class CheetahLong
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ref CheetahCommandMeta meta, ref CheetahObjectId objectId, ushort fieldId, long value);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_long_value_listener")]
        public static extern bool SetListener(Listener listener);


        /// <summary>
        /// Установить значение
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="value"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_long_value")]
        public static extern bool Set(ref CheetahObjectId objectId, ushort fieldId, long value);

        /// <summary>
        /// Инкрементация значения
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="value"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "inc_long_value")]
        public static extern bool Increment(ref CheetahObjectId objectId, ushort fieldId, long increment);


        /// <summary>
        ///  CompareAndSet  
        /// </summary>
        /// <param name="objectId"></param>
        /// <param name="fieldId"></param>
        /// <param name="currentValue">требуемое значение</param>
        /// <param name="newValue">новое значение, устанавливается если текущее равное currentValue</param>
        /// <param name="resetValue">значение, устанавливаемое если пользователь вышел, применимо если команда смогла установить newValue</param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "compare_and_set_long_value")]
        public static extern bool CompareAndSet(ref CheetahObjectId objectId, ushort fieldId, long currentValue, long newValue, long resetValue);
    }
}