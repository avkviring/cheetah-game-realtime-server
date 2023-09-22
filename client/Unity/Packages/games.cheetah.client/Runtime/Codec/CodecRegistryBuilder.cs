using System;
using System.Collections.Generic;
using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Codec.Standard;

namespace Games.Cheetah.Client.Codec
{
    public class CodecRegistryBuilder
    {
        internal static readonly Dictionary<Type, object> DefaultFactories = new();

        internal readonly Dictionary<Type, object> factories = new();

        public delegate Codec<T> CodecFactory<T>(CodecRegistry registry);

        static CodecRegistryBuilder()
        {
            RegisterDefault(_ => new Vector2Codec());
            RegisterDefault(_ => new Vector3Codec());
            RegisterDefault(_ => new CheetahObjectIdCodec());
            RegisterDefault(_ => new ColorCodec());
            RegisterDefault(_ => new StringReferenceCodec());
            RegisterDefault(_ => BoolFormatter.Instance);
            RegisterDefault(_ => ByteFormatter.Instance);
            RegisterDefault(_ => ShortFormatter.Instance);
            RegisterDefault(_ => UShortFormatter.Instance);
            RegisterDefault(_ => IntFormatter.Instance);
            RegisterDefault(_ => UIntFormatter.Instance);
            RegisterDefault(_ => LongFormatter.Instance);
            RegisterDefault(_ => ULongFormatter.Instance);
            RegisterDefault(_ => FloatFormatter.Instance);
            RegisterDefault(_ => DoubleFormatter.Instance);
        }


        public static void ClearDefaultRegistration()
        {
            DefaultFactories.Clear();
        }
        
        
        public static void RegisterDefault<T>(CodecFactory<T> factory)
        {
            var type = typeof(T);
            RemoveExistDefaultFactoryForDisableDomainReloadSupport(type);
            DefaultFactories.Add(type, factory);
        }

        private static void RemoveExistDefaultFactoryForDisableDomainReloadSupport(Type type)
        {
            DefaultFactories.Remove(type);
        }

        public CodecRegistryBuilder Register<T>(CodecFactory<T> factory)
        {
            factories.Add(typeof(T), factory);
            return this;
        }

        public CodecRegistry Build()
        {
            return new CodecRegistry(this);
        }
    }
}