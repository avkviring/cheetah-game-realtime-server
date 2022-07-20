using System;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Internal
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

        internal ref T Add()
        {
            if (Count == items.Length - 1)
            {
                var newItems = new T[(int)(items.Length * 1.5)];
                Array.Copy(items, newItems, items.Length);
                items = newItems;
            }

            return ref items[Count++];
        }

        internal void Clear()
        {
            Count = 0;
        }

        public ref T GetItem(int index)
        {
            return ref items[index];
        }
    }
}