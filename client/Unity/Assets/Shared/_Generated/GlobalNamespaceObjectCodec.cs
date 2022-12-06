using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using UnityEngine;

	// warning warning warning warning warning
	// Code generated by Cheetah relay codec generator - DO NOT EDIT
	// warning warning warning warning warning
	public class GlobalNamespaceObjectCodec:Codec<GlobalNamespaceObject>
	{
		public void Decode(ref CheetahBuffer buffer, ref GlobalNamespaceObject dest)
		{
			dest.field = IntFormatter.Instance.Read(ref buffer);
		}
	
		public void  Encode(in GlobalNamespaceObject source, ref CheetahBuffer buffer)
		{
			IntFormatter.Instance.Write(source.field,ref buffer);
		}
	
	
		[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
		private static void OnRuntimeMethodLoad()
		{
			CodecRegistryBuilder.RegisterDefault(factory=>new GlobalNamespaceObjectCodec());
		}
	
	}
	
