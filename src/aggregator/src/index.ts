import express, { Request, Response } from 'express';
import axios from 'axios';

/**
 * Reference aggregator service.
 *
 * In a production deployment, this service fetches the latest global
 * weight vector from the chain, selects top miners, forwards queries
 * to those miners, and aggregates their responses.  This reference
 * implementation uses a hard‑coded list of miners and simply forwards
 * the query to all of them, returning the first response.
 */

const app = express();
app.use(express.json());

// Hard‑coded miner endpoints for demonstration.  Real implementation
// should discover miners via the chain/state and weight vectors.
const miners: string[] = [process.env.MINER_ENDPOINT || 'http://127.0.0.1:5000'];

app.post('/v1/subnets/:id/query', async (req: Request, res: Response) => {
  const { id } = req.params;
  const { input } = req.body;
  if (!input) {
    return res.status(400).json({ error: 'Missing input' });
  }

  for (const miner of miners) {
    try {
      const response = await axios.post(miner, { input });
      return res.json({ subnet: id, result: response.data.output });
    } catch (error) {
      console.warn(`Failed to query miner ${miner}:`, (error as any).message);
    }
  }

  return res.status(502).json({ error: 'No miners available' });
});

const port = process.env.PORT || 3000;
app.listen(port, () => {
  console.log(`NeuroMesh aggregator listening on port ${port}`);
});