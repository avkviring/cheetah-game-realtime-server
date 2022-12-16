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
        private readonly Dictionary<FieldId, IDisposable> fieldListeners = new();

        public Reader(CheetahClient client)
        {
            this.client = client;
        }

        public IReadOnlyList<CheetahObjectConstructor> GetCreatedObjects(ushort template)
        {
            var listener = GetOrCreateListener<CreatedObjectByTemplateIncomeCommands, ushort, ushort>(createObjectListeners, template,
                CreateObjectsCollectorFactory<CreatedObjectByTemplateIncomeCommands>);
            return listener.GetStream();
        }

        public ReadonlyReferenceList<CheetahObject> GetDeletedObjects(ushort template)
        {
            var listener = GetOrCreateListener<DeletedObjectByTemplateIncomeCommands, ushort, ushort>(deleteObjectListeners, template,
                DeleteObjectsCollectorFactory<DeletedObjectByTemplateIncomeCommands>);
            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<T>.Item> GetModifiedStructures<T>(FieldId.Structure field)
            where T : struct
        {
            var listener = GetOrCreateListener<StructureIncomeByFieldCommandCollector<T>, FieldId, FieldId.Structure>(fieldListeners, field,
                StructureCollectorFactory<T>);

            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<long>.Item> GetModifiedLongs(FieldId.Long field)
        {
            var listener = GetOrCreateListener<LongIncomeByFieldCommandCollector, FieldId, FieldId.Long>(fieldListeners, field, LongCollectorFactory);

            return listener.GetStream();
        }

        public ReadonlyReferenceList<AbstractIncomeByFieldCommandCollector<double>.Item> GetModifiedDoubles(FieldId.Double field)
        {
            var listener = GetOrCreateListener<DoubleIncomeByFieldCommandCollector, FieldId, FieldId.Double>(fieldListeners, field,
                DoubleCollectorFactory);

            return listener.GetStream();
        }

        private T GetOrCreateListener<T, K, F>(
            Dictionary<K, IDisposable> listeners,
            K key,
            Func<CheetahClient, F, IDisposable> factory)
            where T : class
            where F : K
        {
            if (!listeners.TryGetValue(key, out var listener))
            {
                listener = factory(client, (F)key);
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

        private static IDisposable StructureCollectorFactory<T>(CheetahClient client, FieldId.Structure field)
        {
            return new StructureIncomeByFieldCommandCollector<T>(client, field);
        }

        private static IDisposable LongCollectorFactory(CheetahClient client, FieldId.Long field)
        {
            return new LongIncomeByFieldCommandCollector(client, field);
        }

        private static IDisposable DoubleCollectorFactory(CheetahClient client, FieldId.Double field)
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