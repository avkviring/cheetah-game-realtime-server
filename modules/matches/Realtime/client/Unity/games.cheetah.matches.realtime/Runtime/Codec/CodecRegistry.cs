using System;
using System.Collections.Generic;
using System.Reflection;

namespace Cheetah.Matches.Realtime.Codec
{
    public class CodecRegistry
    {
        private readonly Dictionary<Type, object> codecs;

        internal CodecRegistry(CodecRegistryBuilder builder)
        {
            codecs = new Dictionary<Type, object>();
            var factories = new Dictionary<Type, object>(CodecRegistryBuilder.DefaultFactories);
            foreach (var pair in builder.factories)
            {
                factories.Add(pair.Key, pair.Value);
            }

            CreateCodecs(factories);
        }

        private void CreateCodecs(Dictionary<Type, object> factories)
        {
            var current = new Dictionary<Type, object>(factories);
            var createdInCycle = true;
            var lastCycle = false;
            while (current.Count > 0)
            {
                if (!createdInCycle)
                {
                    lastCycle = true;
                }

                createdInCycle = false;
                var next = new Dictionary<Type, object>();
                foreach (var pair in current)
                {
                    try
                    {
                        CreateCodec(pair);
                        createdInCycle = true;
                    }
                    catch (CodecNotFoundException)
                    {
                        if (lastCycle) throw;
                        next.Add(pair.Key, pair.Value);
                    }
                }

                current.Clear();
                foreach (var pair in next)
                {
                    current.Add(pair.Key, pair.Value);
                }
            }
        }

        private void CreateCodec(KeyValuePair<Type, object> item)
        {
            try
            {
                var type = item.Key;
                var method = item.Value.GetType().GetMethod("Invoke");
                codecs[type] = method.Invoke(item.Value, new[] { this });
            }
            catch (TargetInvocationException e)
            {
                throw e.InnerException;
            }
        }

        public Codec<T> GetCodec<T>()
        {
            try
            {
                return (Codec<T>)codecs[typeof(T)];
            }
            catch (KeyNotFoundException)
            {
                throw new CodecNotFoundException("Codec not found, type = " + typeof(T).FullName);
            }
        }
    }

    public class CodecNotFoundException : Exception
    {
        public CodecNotFoundException(string message) : base(message)
        {
        }
    }
}