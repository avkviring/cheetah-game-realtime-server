using System.Runtime.InteropServices;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class FieldFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void Listener(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, FieldType fieldType);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_field_listener")]
        public static extern byte SetListener(ushort clientId, Listener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_field")]
        public static extern byte Delete(ushort clientId, in CheetahObjectId objectId, ushort fieldId, FieldType value);
    }
}