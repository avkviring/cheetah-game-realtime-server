using System.Collections;
using Cheetah.Matches.Relay.Editor.GRPC;

namespace Cheetah.Matches.Relay.Editor.DumpViewer.TypesExtension
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