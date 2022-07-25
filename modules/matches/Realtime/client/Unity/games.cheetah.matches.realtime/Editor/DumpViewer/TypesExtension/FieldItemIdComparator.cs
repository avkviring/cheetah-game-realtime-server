using System.Collections;
using Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.TypesExtension
{
    public class FieldItemIdComparator : IComparer
    {
        public int Compare(object x, object y)
        {
            var f1 = (ObjectsViewer.FieldItem)x;
            var f2 = (ObjectsViewer.FieldItem)y;
            return f1.Id.CompareTo(f2.Id);
        }
    }
}