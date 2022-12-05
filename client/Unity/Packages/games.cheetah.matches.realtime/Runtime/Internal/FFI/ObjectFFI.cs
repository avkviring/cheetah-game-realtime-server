using System.Runtime.InteropServices;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal.FFI
{
    internal static class ObjectFFI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreateListener(in CheetahObjectId objectId, ushort template);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreatedListener(in CheetahObjectId objectId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_create_object_listener")]
        public static extern byte SetCreateListener(ushort clientId, CreateListener listener);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_created_object_listener")]
        public static extern byte SetCreatedListener(ushort clientId, CreatedListener listener);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_object")]
        public static extern byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref CheetahObjectId objectId);
        
        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "created_object")]
        public static extern byte Created(ushort clientId, in CheetahObjectId objectId, bool roomOwner, ref CheetahBuffer singletonKey);


        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DeleteListener(in CheetahObjectId objectId);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_delete_object_listener")]
        public static extern byte SetDeleteListener(ushort clientId, DeleteListener objectDeleteListener);


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "delete_object")]
        public static extern byte Delete(ushort clientId, in CheetahObjectId objectId);
    }
}