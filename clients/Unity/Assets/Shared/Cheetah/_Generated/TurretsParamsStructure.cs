using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Types;
using UnityEngine;
using Shared.Types;

// ReSharper disable once CheckNamespace
namespace Shared.Types
{
		public class TurretsParamsStructureCodec:Codec<TurretsParamsStructure>
		{
			public void Decode(ref CheetahBuffer buffer, ref TurretsParamsStructure dest)
			{
				dest.Speed = PrimitiveReaders.ReadDouble(ref buffer);
				dest.Damage = PrimitiveReaders.ReadDouble(ref buffer);
			}
	
			public void  Encode(ref TurretsParamsStructure source, ref CheetahBuffer buffer)
			{
				PrimitiveWriters.Write(source.Speed,ref buffer);
				PrimitiveWriters.Write(source.Damage,ref buffer);
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new TurretsParamsStructureCodec());
			}
	
		}
	
	
}
