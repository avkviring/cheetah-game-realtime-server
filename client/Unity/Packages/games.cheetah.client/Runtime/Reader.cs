using System;
using System.Collections.Generic;
using System.Linq;
using Games.Cheetah.Client.DOA.Income.ByField;
using Games.Cheetah.Client.DOA.Income.ByTemplate;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client
{
    public class Reader 
    {
        private readonly CheetahClient client;
        private readonly Dictionary<ushort, IDisposable> createObjectListeners = new();
        private readonly Dictionary<ushort, IDisposable> deleteObjectListeners = new();
        private readonly Dictionary<ushort, IDisposable> fieldListeners = new();

        public Reader(CheetahClient client)
        {
            this.client = client;
        }

        public IReadOnlyList<CheetahObjectConstructor> GetCreatedObjects(ushort template)
        {
            var listener = GetOrCreateListener<CreatedObjectByTemplateIncomeCommands>(createObjectListeners, template,
                CreateObjectsCollectorFactory<CreatedObjectByTemplateIncomeCommands>);
            return listener.GetStream();
        }

        public ReadonlyReferenceList<CheetahObject> GetDeletedObjects(ushort template)
        {
            var listener = GetOrCreateListener<DeletedObjectByTemplateIncomeCommands>(deleteObjectListeners, template,
                DeleteObjectsCollectorFactory<DeletedObjectByTemplateIncomeCommands>);
            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<T>.Item> GetModifiedStructures<T>(ushort field)
            where T : struct
        {
            var listener = GetOrCreateListener<StructureIncomeByFieldCommandCollector<T>>(fieldListeners, field,
                StructureCollectorFactory<T>);

            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<long>.Item> GetModifiedLongs(ushort field)
        {
            var listener = GetOrCreateListener<LongIncomeByFieldCommandCollector>(fieldListeners, field,
                LongCollectorFactory);

            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<double>.Item> GetModifiedDoubles(ushort field)
        {
            var listener = GetOrCreateListener<DoubleIncomeByFieldCommandCollector>(fieldListeners, field,
                DoubleCollectorFactory);

            return listener.GetStream();
        }

        private T GetOrCreateListener<T>(Dictionary<ushort, IDisposable> listeners, ushort key,
            Func<CheetahClient, ushort, IDisposable> factory) where T : class
        {
            if (!listeners.TryGetValue(key, out var listener))
            {
                listener = factory(client, key);
                listeners[key] = listener;
            }

            return listener as T;
        }

        private static IDisposable CreateObjectsCollectorFactory<T>(CheetahClient client,
            ushort field)
        {
            return new CreatedObjectByTemplateIncomeCommands(client, field);
        }

        private static IDisposable DeleteObjectsCollectorFactory<T>(CheetahClient client,
            ushort field)
        {
            return new DeletedObjectByTemplateIncomeCommands(client, field);
        }

        private static IDisposable StructureCollectorFactory<T>(CheetahClient client,
            ushort field)
        {
            return new StructureIncomeByFieldCommandCollector<T>(client, field);
        }

        private static IDisposable LongCollectorFactory(CheetahClient client,
            ushort field)
        {
            return new LongIncomeByFieldCommandCollector(client, field);
        }

        private static IDisposable DoubleCollectorFactory(CheetahClient client,
            ushort field)
        {
            return new DoubleIncomeByFieldCommandCollector(client, field);
        }

        public void Dispose()
        {
            var disposables = fieldListeners.Values
                .Concat(createObjectListeners.Values)
                .Concat(deleteObjectListeners.Values);
            foreach (var disposable in disposables)
            {
                disposable.Dispose();
            }
        }
    }
}