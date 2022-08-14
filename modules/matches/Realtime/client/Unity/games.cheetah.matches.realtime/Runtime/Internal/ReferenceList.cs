using System;
using Cheetah.Matches.Realtime.Types;

namespace Cheetah.Matches.Realtime.Internal
{
    public class ReferenceList<T> : ReadonlyReferenceList<T> where T : struct
    {
        public int Count { get; private set; }
        private T[] items;


        public ReferenceList(int capacity)
        {
            items = new T[capacity];
        }

        public ReferenceList() : this(64)
        {
        }

        public ref T Add()
        {
            if (Count == items.Length - 1)
            {
                var newItems = new T[(int)(items.Length * 1.5)];
                Array.Copy(items, newItems, items.Length);
                items = newItems;
            }

            return ref items[Count++];
        }

        public void Clear()
        {
            Count = 0;
        }

        public ref T GetItem(int index)
        {
            return ref items[index];
        }
    }
}