using System.Runtime.InteropServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI
{
    public interface IObjectServerAPI
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreateListener(in CheetahObjectId objectId, ushort template);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void CreatedListener(in CheetahObjectId objectId);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DeleteListener(in CheetahObjectId objectId);

        byte SetCreateListener(ushort clientId, CreateListener listener);
        byte SetCreatedListener(ushort clientId, CreatedListener listener);
        byte CreateObject(ushort clientId, ushort template, ulong accessGroup, ref CheetahObjectId objectId);
        byte CreatedObject(ushort clientId, in CheetahObjectId objectId, bool roomOwner, ref CheetahBuffer singletonKey);
        byte SetDeleteListener(ushort clientId, DeleteListener objectDeleteListener);
        byte Delete(ushort clientId, in CheetahObjectId objectId);
    }
}