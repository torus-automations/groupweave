import { Evm } from '../../utils/ethereum';

const contractId = process.env.NEXT_PUBLIC_contractId;

export default async function handler(req, res) {
    try {
        const { address: senderAddress } = await Evm.deriveAddressAndPublicKey(
            contractId,
            "ethereum-1",
          );
        res.status(200).json({ senderAddress });
    } catch (error) {
        console.log('Error getting worker account:', error);
        res.status(500).json({ error: 'Failed to get worker account' });
    }
} 