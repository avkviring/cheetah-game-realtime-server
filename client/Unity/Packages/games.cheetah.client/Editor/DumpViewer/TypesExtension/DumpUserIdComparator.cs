using System.Collections;
using Games.Cheetah.GRPC.Admin;

namespace Games.Cheetah.Client.Editor.DumpViewer.TypesExtension
{
    internal class DumpUserIdComparator : IComparer
    {
        public int Compare(object x, object y)
        {
            var o1 = (DumpUser)x;
            var o2 = (DumpUser)y;
            return o1.Id.CompareTo(o2.Id);
        }
    }
}