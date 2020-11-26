using System.Runtime.InteropServices;
using System.Text;

namespace CheetahRelay
{
    public static class CreateObjectCommand
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ref CommandMeta meta, ref RelayObjectId objectId, ushort template, ref GameObjectFields fields);

        /// <summary>
        /// Установить обработчик серверных команд для текущего клиента
        /// </summary>
        /// <param name="listener"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_create_object_listener")]
        public static extern bool SetListener(Listener listener);

        /// <summary>
        /// Создать объект
        /// </summary>
        /// <param name="template"></param>
        /// <param name="accessGroup"></param>
        /// <param name="fields"></param>
        /// <param name="objectId"></param>
        /// <returns>false - клиент не найден</returns>
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_object")]
        public static extern bool Create(ushort template, ulong accessGroup, ref GameObjectFields fields, out RelayObjectId objectId);
    }


    [StructLayout(LayoutKind.Sequential)]
    public struct GameObjectFields
    {
        public Structures structures;
        public DoubleValues doubles;
        public LongValues longs;

        public override string ToString()
        {
            var result = new StringBuilder();
            result.AppendLine("GameObjectFields(");
            result.AppendLine(structures.ToString());
            result.AppendLine(doubles.ToString());
            result.AppendLine(longs.ToString());
            result.AppendLine(")");
            return result.ToString();
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct Structures
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed ushort fields[Const.MaxFieldsInObject];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed byte sizes[Const.MaxFieldsInObject];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.AllStructuresSize)]
        public fixed byte values[Const.AllStructuresSize];

        public override string ToString()
        {
            var result = new StringBuilder();
            result.AppendLine("Structures(");
            for (var i = 0; i < count; i++)
            {
                result.AppendLine("size [" + fields[i] + "]=" + sizes[i]);
            }

            result.AppendLine(")");
            return result.ToString();
        }
    }


    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct LongValues
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed ushort fields[Const.MaxFieldsInObject];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed long values[Const.MaxFieldsInObject];
        
        public override string ToString()
        {
            var result = new StringBuilder();
            result.AppendLine("LongValues(");
            for (var i = 0; i < count; i++)
            {
                result.AppendLine("[" + fields[i] + "]=" + values[i]);
            }

            result.AppendLine(")");
            return result.ToString();
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct DoubleValues
    {
        public byte count;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed ushort fields[Const.MaxFieldsInObject];

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxFieldsInObject)]
        public fixed double values[Const.MaxFieldsInObject];
        
        public override string ToString()
        {
            var result = new StringBuilder();
            result.AppendLine("DoubleValues(");
            for (var i = 0; i < count; i++)
            {
                result.AppendLine("[" + fields[i] + "]=" + values[i]);
            }

            result.AppendLine(")");
            return result.ToString();
        }
    }
}