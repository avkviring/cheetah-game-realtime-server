using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;
using UnityEngine;
using Shared.Types;

// ReSharper disable once CheckNamespace
namespace Shared_Types
{
		// warning warning warning warning warning
		// Code generated by Cheetah relay codec generator - DO NOT EDIT
		// warning warning warning warning warning
		public class SomeSingletonKeyCodec:Codec<Shared.Types.SomeSingletonKey>
		{
			public void Decode(ref NetworkBuffer buffer, ref Shared.Types.SomeSingletonKey dest)
			{
				dest.Key = IntFormatter.Instance.Read(ref buffer);
			}
	
			public void  Encode(in Shared.Types.SomeSingletonKey source, ref NetworkBuffer buffer)
			{
				IntFormatter.Instance.Write(source.Key,ref buffer);
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			private static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new SomeSingletonKeyCodec());
			}
	
		}
	
	
}
