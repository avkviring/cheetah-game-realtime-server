using System;
using Cheetah.Matches.Relay.Internal;
using Cheetah.Matches.Relay.Types;

namespace Cheetah.Matches.Relay.Income.ByObject
{
    public class AbstractIncomeByObjectCommandCollector<T> : IDisposable
    {
        internal readonly ReferenceList<Item> stream = new ReferenceList<Item>(2);
        protected readonly ushort fieldId;
        protected readonly CheetahClient client;


        public struct Item
        {
            public ushort commandCreator;
            public T value;
        }

        protected AbstractIncomeByObjectCommandCollector(CheetahClient client, ushort fieldId)
        {
            this.client = client;
            this.fieldId = fieldId;
            this.client.BeforeUpdateHook += OnBeforeUpdate;
        }

        private void OnBeforeUpdate()
        {
            stream.Clear();
        }

        public ReadonlyReferenceList<Item> GetStream()
        {
            return stream;
        }

        public virtual void Dispose()
        {
            client.BeforeUpdateHook -= OnBeforeUpdate;
        }
    }
}